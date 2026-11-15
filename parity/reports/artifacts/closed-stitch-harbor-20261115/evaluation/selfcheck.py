"""Static analytic/mutation checks only; never runs an engine or garment."""
import argparse
import ast
import copy
import json
from pathlib import Path
import sys
import tempfile
import time

sys.dont_write_bytecode = True
import capsule_geometry as capsule
import decimal_oracle
import evaluator as e


def refused(call, contains=None):
    try:
        call()
    except (ValueError, AssertionError, KeyError, TypeError, OverflowError, FileExistsError) as error:
        if contains is not None:
            assert contains in str(error), (contains, str(error))
        return
    raise AssertionError("Invalid synthetic input accepted")


def function_ast(path, name):
    return next(node for node in ast.parse(path.read_text()).body if isinstance(node, ast.FunctionDef) and node.name == name)


def main(output):
    started = time.monotonic()
    source_names = [p.name for p in e.ROOT.glob("*.py")]+["METHOD.md", "README.md", "NUMERIC-PROVENANCE.json", "REST-BINDING.json", "READONLY-INPUT-BINDING.json"]
    source_hashes = {name: e.sha(e.ROOT/name) for name in sorted(source_names)}
    checks = []
    preparation = e.bindings.verify_static(require_freeze=False)
    prior = e.verify_readonly_inputs()
    checks.append("Exact prospective source45/input2 binding, unchanged numerical modules and prior immutable evidence; producer remains a separate unexecuted prerequisite")
    for name, item in e.read(e.ROOT / "NUMERIC-PROVENANCE.json").items():
        source, actual = e.REPO / item["source"], e.ROOT / name
        assert e.sha(source) == item["source_sha256"]
        assert e.sha(actual) == item["copy_sha256"]
        if isinstance(item["exact_functions"], list):
            for function in item["exact_functions"]:
                assert ast.dump(function_ast(source, function)) == ast.dump(function_ast(actual, function))
        else:
            assert source.read_bytes() == actual.read_bytes()
    e.metrics.self_check()
    checks.append("Prior independent metrics byte-identical; exact originalclosest-feature/segment/100digitQP functionASTs; inherited analytic/pruning/OBJchecks passed")
    assert not e.same(0., -0.) and not e.same(True, 1) and not e.same(1, 1.)
    for text in ('{"x":1,"x":2}', '{"x":{"y":0,"y":1}}', '{"x":NaN}', '{"x":Infinity}', '{"x":1e1000}', '{"x":'+('9'*1000)+'}'):
        refused(lambda text=text: e.strict_loads(text))
    assert not e.finite_vectors([[True, 0., 0.]], 1)
    assert all(e.screen(.149, .004, .001, 0., True, 0).values())
    for values in ((.15,.004,.001,0.,True,0), (.149,.005,.001,0.,True,0), (.149,.004,.002,0.,True,0),
                   (.149,.004,.001,1e-16,True,0), (.149,.004,.001,0.,False,0), (.149,.004,.001,0.,True,1)):
        assert not all(e.screen(*values).values())
    assert not all(e.screen(.149, .004, .001, 0., True, 0, 1).values())
    assert not all(e.screen(.149, .004, .001, 0., True, False, 0).values())
    checks.append("Duplicate/nestedduplicate/nonfinite/overflow JSON rejected; signedzero identity and strict15percent/5mm/2mm/targetzero/finite/uncertaintyzero boundaries preserved")
    for schema, attempt in ((True, 1), (1., 1), (1, True), (1, 1.)):
        refused(lambda schema=schema, attempt=attempt: e.receipt_check({"schema": schema, "case": "base", "attempt": attempt}, "base", Path("unused"), {}))
    cases = e.read(e.REPO / ".local/capsule-independent-rust-20261115/inputs.json")
    expected = e.read(e.REPO / ".local/capsule-independent-rust-20261115/expected.json")
    actual83 = e.read(e.REPO / ".local/capsule-independent-rust-20261115/RESULT.json")
    assert actual83["status"] == "passed_all_83_static_cases" and not actual83["failures"]
    results = []
    assert len(cases) == len(expected) == 83
    for value, reference in zip(cases, expected):
        assert value["label"] == reference["label"]
        independent = capsule.distance_interval(value["triangle"], {"a": value["axis_a"], "b": value["axis_b"]})
        assert not independent["uncertain"], (value["label"], independent)
        distance, tolerance = reference["independent_qp_distance"], reference["tolerance"]
        assert independent["distance_lower_m"] <= distance+tolerance
        assert independent["distance_upper_m"] >= distance-tolerance
        assert independent["distance_upper_m"]-independent["distance_lower_m"] <= tolerance*4, (value["label"], independent, reference)
        recomputed = decimal_oracle.independent_qp(value["triangle"], value["axis_a"], value["axis_b"])
        assert abs(recomputed-distance) <= tolerance
        results.append({"label": value["label"], "independent": independent, "original100digit_distance": distance, "original_tolerance_unchanged": tolerance})
    checks.append("All83 retained analytic/random/parallel/skinny/line/point/scale/Harborwitness cases crosschecked against unchanged100digit independentoracle and savedactualRust83report; no binary executed")
    thin = [[-1., 0., 0.], [1., 0., 0.], [0., 1e-7, 0.]]
    axis = {"a": [0., 5e-8, 0.], "b": [0., 5e-8, 0.]}
    unresolved = capsule.distance_interval(thin, axis, allow_fallback=False)
    assert unresolved["uncertain"] and unresolved["distance_lower_m"] == 0.
    resolved = capsule.distance_interval(thin, axis)
    assert not resolved["uncertain"] and resolved["distance_upper_m"] < 1e-50
    for height in (1e-5, 1e-7, 1e-9, 1e-12):
        tri = [[-1., 0., 0.], [1., 0., 0.], [0., height, 0.]]
        row = capsule.high_precision_interval(tri, [0., height/2, 0.], [0., height/2, 0.])
        assert not row["uncertain"] and row["distance_lower_m"] == 0. and row["distance_upper_m"] < 1e-50
    huge_offset = [[1e300, 0., 0.], [1e300, 1e-100, 0.], [1e300, 0., 1e-100]]
    tiny_axis = {"a": [1e300, 2.5e-101, 2.5e-101], "b": [1e300, 2.5e-101, 2.5e-101]}
    huge_unknown = capsule.distance_interval(huge_offset, tiny_axis, allow_fallback=False)
    assert huge_unknown["uncertain"] and huge_unknown["distance_upper_m"] is None
    huge_resolved = capsule.distance_interval(huge_offset, tiny_axis)
    assert not huge_resolved["uncertain"] and huge_resolved["distance_upper_m"] < 1e-150
    checks.append("Unresolvedskinny face occupancy remainsUNKNOWN withoutfallback;100digit primal/dual face-interior checks certify nonzeroarea altitudes1e-5..1e-12 without edge-only omission")
    material = {"id": 1, "collision_thickness": .003, "areal_density": .2, "warp_stiffness": 400., "weft_stiffness": 400., "shear_stiffness": 100.}
    def synthetic(triangle, uv, axis_a, axis_b, raw_initial, raw_uncertain):
        meta = {"project": {"pieces": [{"id": 1, "material": 1, "grain_angle": 0.}], "materials": [material],
                            "mannequin": {"capsules": [{"a": axis_a, "b": axis_b, "radius": .05}]}},
                "piece_ids": [1, 1, 1], "triangles": [[0, 1, 2]], "uv_m": uv, "rest_positions_m": triangle, "stitches": []}
        meta["masses_kg"] = e.metrics.masses(meta)
        strain = e.metrics.measure(meta, triangle)["peak"]
        contact = capsule.measure(meta, triangle)
        d = {"finite": True, "simulation_time": 0., "seam_assembly_remaining": 0., "max_stretch_error": strain["absolute_strain"],
             "max_seam_error": 0., "max_contact_error": (contact["penetration_lower_m"]+contact["penetration_upper_m"])/2,
             "kinetic_energy": 0., "peak_strain": {"triangle": strain["triangle_index"], "axis": strain["axis"], "signed_strain": strain["signed_strain"]},
             "contacts": 0, "initial_intersections": raw_initial, "capsule_surface_uncertain_pairs": raw_uncertain}
        state = {"global_frame": 0, "settings_last_step": e.settings(0), "phase": "assembly", "producer_wall_seconds": 0.,
                 "positions_m": triangle, "velocities_m_per_s": [[0., 0., 0.]]*3, "diagnostics": d}
        row = e.measure_frame(meta, state, 0, None, [], 0., e.metrics.exclusion(meta))
        return row
    skinny_uv = [[0., 0.], [2., 0.], [1., 1e-7]]
    skinny_world = [[-1., 2., 0.], [1., 2., 0.], [0., 2.+1e-7, 0.]]
    policy_unknown = synthetic(skinny_world, skinny_uv, [0., 2., 10.], [0., 2., 10.], 0, 0)
    assert policy_unknown["independent_unknown_capsule_pairs"] == 0 and policy_unknown["reported_conservative_capsule_pairs"] == 0
    assert policy_unknown["quality_gates"]["physical_contact_below_0_002_m"] and not policy_unknown["required_diagnostic_verification"]["product_uncertainty_policy_count_agreement"]
    reported_nonzero = synthetic(skinny_world, skinny_uv, [0., 2., 10.], [0., 2., 10.], 0, 1)
    assert not reported_nonzero["quality_gates"]["reported_capsule_conservative_pairs_exact_zero"]
    assert reported_nonzero["quality_gates"]["independent_capsule_unknown_pairs_exact_zero"]
    assert reported_nonzero["quality_gates"]["physical_contact_below_0_002_m"]
    conservative_initial = synthetic(skinny_world, skinny_uv, [0., 2., 10.], [0., 2., 10.], 1, 1)
    assert conservative_initial["initial_intersections_independent_range"]["upper"] == 0
    assert conservative_initial["initial_intersection_count_status"] == "unverified_conservative_policy_may_add_intersections"
    assert conservative_initial["initial_intersections_expected_producer_count_range"] is None
    assert "independent initial vertex/self/surface intersection count mismatch" not in conservative_initial["issues"]
    assert not conservative_initial["required_diagnostic_verification"]["initial_intersection_count_agreement"]
    face = [[-1., 2., -1.], [1., 2., -1.], [0., 2., 1.]]
    uv = [[-1., -1.], [1., -1.], [0., 1.]]
    full_count = synthetic(face, uv, [0., 1., 0.], [0., 3., 0.], 1, 0)
    assert full_count["initial_intersections_independent_range"] == {"lower": 1, "upper": 1, "vertex_supported": 0, "capsule_threshold_ambiguous_pairs": 0}
    assert full_count["required_diagnostic_verification"]["initial_intersection_count_agreement"]
    missing_surface = synthetic(face, uv, [0., 1., 0.], [0., 3., 0.], 0, 0)
    assert "independent initial vertex/self/surface intersection count mismatch" in missing_surface["issues"]
    threshold = 2.+(.05+.003)-1e-8
    ambiguous = synthetic(face, uv, [0., threshold, 0.], [0., threshold, 0.], 0, 0)
    assert ambiguous["initial_intersections_independent_range"]["capsule_threshold_ambiguous_pairs"] == 1
    assert ambiguous["initial_intersection_count_status"] == "unknown_ambiguous_threshold_or_geometry"
    assert "independent initial vertex/self/surface intersection count mismatch" not in ambiguous["issues"]
    invalid_count = synthetic(face, uv, [0., 1., 0.], [0., 3., 0.], True, False)
    assert "invalid diagnostic initial-intersection count" in invalid_count["issues"]
    assert "invalid diagnostic conservative capsule-pair count" in invalid_count["issues"]
    checks.append("Rawconservative-policy counts and oracleunknowns staydistinct including separatedskinny pair; allpair wellconditionedzero proof; face-only initialintersectionincluded; thresholdambiguityUNKNOWN; booleancounts rejected")
    retained = []
    with tempfile.TemporaryDirectory(prefix="synthetic-selfcheck-", dir=e.ROOT) as temp:
        temp = Path(temp)
        report = temp / "report.json"
        e.write_report(report, {"preserve": True})
        refused(lambda: e.write_report(report, {"preserve": False}))
        assert e.read(report) == {"preserve": True}
        link = temp / "link.json"
        link.symlink_to(report)
        refused(lambda: e.safe_file(link, temp))
        refused(lambda: e.safe_file("../report.json", temp))
        auth = temp / "not-authorized.json"
        auth.write_text(json.dumps({"schema": 1, "status": "preparation"}))
        refused(lambda: e.authorization(auth))
        original_safe_file = e.safe_file
        try:
            e.safe_file = lambda path, base: Path(path) if Path(path).is_absolute() else Path(base)/path
            for schema, attempts in ((True, 1), (1., 1), (1, True), (1, 1.)):
                auth.write_text(json.dumps({"schema": schema, "status": "authorized", "source_commit": e.SOURCE,
                                           "cases": ["base", "height"], "attempts_per_case": attempts}))
                refused(lambda: e.authorization(auth))
            auth_value = {"schema": 1, "status": "authorized", "source_commit": e.SOURCE,
                          "cases": ["base", "height"], "attempts_per_case": 1,
                          "explicit_root_authorization": True,
                          "runner": {"path": str((e.EXPERIMENT / "runner.py").relative_to(e.REPO)), "sha256": "0"*64}}
            auth.write_text(json.dumps(auth_value))
            refused(lambda: e.authorization(auth), "Artifact hash mismatch")
            moved_runner = temp / "moved-runner.py"
            moved_runner.write_bytes((e.EXPERIMENT / "runner.py").read_bytes())
            auth_value["runner"] = {"path": str(moved_runner.relative_to(e.REPO)), "sha256": e.sha(moved_runner)}
            auth.write_text(json.dumps(auth_value))
            refused(lambda: e.authorization(auth), "Wrong actual execution wrapper")
        finally:
            e.safe_file = original_safe_file
        checks.append("Exclusive output cannot overwrite oldreport; symlink/parenttraversal rejected; preparation cannot authorize actualcase analysis")
        for case in e.REVISIONS:
            run = e.GOLD / "runs" / case
            meta = e.read(run / "rest-metadata.json")
            e.check_metadata(meta, case)
            states = [e.strict_loads(line) for line in (run / "trajectory.jsonl").read_text().splitlines()]
            steps = [e.strict_loads(line) for line in (run / "steps.jsonl").read_text().splitlines()]
            assert len(states) == 181 and len(steps) == 180
            target, expected_time = e.initial_targets(meta), 0.
            for frame in range(181):
                if frame:
                    target, expected_time = e.target_step(target), expected_time+1/60
                assert e.same(states[frame]["settings_last_step"], e.settings(frame))
                assert e.same(states[frame]["diagnostics"]["simulation_time"], expected_time)
                assert e.same(states[frame]["diagnostics"]["seam_assembly_remaining"], max(target))
            assert max(target) == 0.
            excluded = e.metrics.exclusion(meta)
            pos = states[180]["positions_m"]
            pruned = capsule.measure(meta, pos)
            exhaustive = capsule.measure(meta, pos, exhaustive=True)
            assert pruned["penetration_lower_m"] == exhaustive["penetration_lower_m"]
            assert pruned["penetration_upper_m"] == exhaustive["penetration_upper_m"]
            assert pruned["worst"] == exhaustive["worst"] and pruned["uncertain_pairs"] == exhaustive["uncertain_pairs"] == 0
            historical = e.HISTORY[f"{case}_frame180_capsule_surface_m"]
            assert pruned["penetration_lower_m"] <= historical <= pruned["penetration_upper_m"]
            assert pruned["penetration_lower_m"] > .002
            for frame in (0, 60, 180):
                obj = e.GOLD / f"prepared/{case}/rest.obj" if frame == 0 else run / f"stage-{frame:03}.obj"
                e.check_obj(meta, states[frame]["positions_m"], obj)
            bad = copy.deepcopy(states[0])
            bad["diagnostics"]["peak_strain"]["triangle"] = (bad["diagnostics"]["peak_strain"]["triangle"]+1) % len(meta["triangles"])
            row = e.measure_frame(meta, bad, 0, None, e.initial_targets(meta), 0., excluded, legacy=True)
            assert not row["exact_attribution_agreement"] and "EXACT diagnostic peak triangle/axis attribution mismatch" in row["issues"]
            bad = copy.deepcopy(states[0])
            bad["settings_last_step"]["gravity"][0] = 1e-13
            bad["diagnostics"]["simulation_time"] = 1e-15
            bad["diagnostics"]["seam_assembly_remaining"] += 1e-15
            row = e.measure_frame(meta, bad, 0, None, e.initial_targets(meta), 0., excluded, legacy=True)
            assert "actual state settings mismatch" in row["issues"]
            assert "exact repeated-dt simulation time recurrence mismatch" in row["issues"]
            assert "exact per-stitch/substep closure recurrence mismatch" in row["issues"]
            bad = copy.deepcopy(states[1])
            bad["diagnostics"]["initial_intersections"] += 1
            row = e.measure_frame(meta, bad, 1, steps[0], e.target_step(e.initial_targets(meta)), 1/60, excluded, legacy=True, initial_intersections=0)
            assert "retained initial-intersection count changed or original count unavailable" in row["issues"]
            bad_meta = copy.deepcopy(meta)
            bad_meta["triangles"][0][0] = float(bad_meta["triangles"][0][0])
            refused(lambda: e.check_metadata(bad_meta, case))
            original = (e.GOLD / f"prepared/{case}/rest.obj").read_text()
            obj = temp / f"{case}.obj"
            for mutated in (original.replace("g piece_2", "g piece_2\ng piece_2", 1), original.replace("vt ", "vt nan ", 1),
                            original.replace("f 1/1/1", "f 0/0/0", 1), original.replace("vn ", "vn 100 ", 1)):
                obj.write_text(mutated)
                refused(lambda: e.check_obj(meta, states[0]["positions_m"], obj))
            retained.append({"case": case, "states": len(states), "steps": len(steps), "rest_metadata_sha256": e.sha(run / "rest-metadata.json"),
                             "trajectory_sha256": e.sha(run / "trajectory.jsonl"), "step_sha256": e.sha(run / "steps.jsonl"),
                             "full_static_capsule_at180": pruned, "historical_failure_retained": True})
        checks.append("Bothold181frame schedules/time/target recurrences exact; actualoldmetadata/inputf32colors/masses bound; sixsame-state OBJchecks and fourmutationclassespercase; exactattribution/settings/time/target mutations remainfailed")
        checks.append("ConservativeAABB full-static capsule pruning equals exhaustive on botholdfinalstates; old9.0097/10.6724mm physicalsurfacefailures retained")
    result = {"status": "passed", "kind": "Static independent evaluator preparation selfchecks", "new_garment_execution": False,
              "new_product_or_engine_execution": False, "new_physics_initializations_or_steps": 0, "execution_authorized": False,
              "preparation": preparation, "prior_immutable_binding": prior, "checks": checks, "oracle83": results, "retained_old_cases": retained,
              "explicit_unresolved_skinny_case": unresolved, "resolved_skinny_case": resolved, "historical_failures": e.HISTORY,
              "huge_common_offset_unknown": huge_unknown, "huge_common_offset_decimal_resolved": huge_resolved,
              "source_hashes": source_hashes, "wall_seconds": time.monotonic()-started, "scope": e.SCOPE}
    e.bindings.verify_static(require_freeze=False)
    e.verify_readonly_inputs()
    assert {name: e.sha(e.ROOT/name) for name in source_hashes} == source_hashes
    e.write_report(output, result)
    print(json.dumps({"status": "passed", "checks": len(checks), "cases": len(results), "new_garment_execution": False, "seconds": result["wall_seconds"]}))


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, required=True)
    main(parser.parse_args().output)
