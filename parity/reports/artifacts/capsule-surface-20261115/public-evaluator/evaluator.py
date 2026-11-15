"""Frozen read-only evaluator for two original Harbor cases; never executes cloth."""
import argparse
import copy
import hashlib
import json
import math
from pathlib import Path
import struct
import sys
import time

sys.dont_write_bytecode = True
import capsule_geometry
import metrics

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parent
EXPERIMENT = REPO / "experiment"
GOLD = REPO / "historical"
SOURCE = "0770204a54abdf5f929b6626b01d7fcfb0e13ca4"
PREVIOUS_PLAN_SHA = "98d575ccfb04afad9ddf72f9ca215fea7161f58ed0de9da1bc44f80e0bebcc12"
PLAN_SHA = "d76f668d21d2837b41574b2d403f7ed6ed24f873265e56cbcbb5e666280034e6"
PRODUCER_BINDING_SHA = "68ca7dcd05f4a443fe71f76fbd0accef9c60249befec4e1638c9065e77534c9a"
REVISIONS = {"base": 20, "height": 17}
CAP_SECONDS = 420.
RSS_CAP = 268435456
OUTPUT_CAP = 268435456
OUTPUT_DIRECTORIES = ("inputs", "preflight-base", "preflight-height", "runs")
SCOPE = "Implementation-only static boundary-state evidence; no equilibrium, calibration, moving avatars, self edge-edge, ellipsoid surface, full CCD, UI acceptance or reference parity."
HISTORY = {"base_frame29_strain": .1535589019880823, "base_frame180_capsule_surface_m": .009009700546250568,
           "height_frame180_capsule_surface_m": .010672413745369775, "status": "Historical failures retained, never replaced by new final-state flags"}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def number(value):
    try:
        return type(value) in (int, float) and math.isfinite(value)
    except OverflowError:
        return False


def strict_loads(text):
    def pairs(items):
        result = {}
        for key, value in items:
            require(key not in result, f"Duplicate external JSON key: {key}")
            result[key] = value
        return result
    def floating(token):
        value = float(token)
        require(math.isfinite(value), "Nonfinite external JSON number")
        return value
    def integer(token):
        value = int(token)
        require(number(value), "Integer outside finite numeric range")
        return value
    return json.loads(text, object_pairs_hook=pairs, parse_float=floating,
                      parse_int=integer,
                      parse_constant=lambda token: require(False, f"Nonfinite external JSON: {token}"))


def read(path):
    return strict_loads(Path(path).read_text())


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def same(a, b):
    if type(a) is float and type(b) is float:
        return struct.pack(">d", a) == struct.pack(">d", b)
    if isinstance(a, list) and isinstance(b, list):
        return len(a) == len(b) and all(same(x, y) for x, y in zip(a, b))
    if isinstance(a, dict) and isinstance(b, dict):
        return a.keys() == b.keys() and all(same(a[k], b[k]) for k in a)
    return type(a) is type(b) and a == b


def write_report(path, value):
    path = Path(path)
    require(path.resolve().is_relative_to(ROOT.resolve()) or path.resolve().is_relative_to(EXPERIMENT.resolve()), "Output outside private experiment")
    require(not path.is_symlink(), "Symlink output rejected")
    encoded = json.dumps(value, indent=2, allow_nan=False)+"\n"
    if path.resolve().is_relative_to(EXPERIMENT / "runs"):
        require(numerical_output_bytes()+len(encoded.encode()) <= OUTPUT_CAP, "Numerical-output256MiB cap exceeded before report write")
    with path.open("x") as stream:
        stream.write(encoded)


def numerical_output_bytes():
    total = 0
    for name in OUTPUT_DIRECTORIES:
        directory = EXPERIMENT / name
        require(not directory.is_symlink(), "Symlink numerical-output directory")
        for path in directory.rglob("*"):
            require(not path.is_symlink(), "Symlink numerical output")
            if path.is_file():
                total += path.stat().st_size
    return total


def safe_file(path, base):
    path, base = Path(path), Path(base).resolve()
    require(".." not in path.parts, "Parent traversal in artifact path")
    if not path.is_absolute():
        path = base / path
    require(path.resolve().is_relative_to(base), "Artifact escaped bound directory")
    current = path
    while current != base:
        require(not current.is_symlink(), "Symlink artifact rejected")
        current = current.parent
    require(path.is_file(), f"Missing artifact: {path.name}")
    return path


def descriptor(value, base):
    require(isinstance(value, dict) and set(value) == {"path", "sha256"}, "Artifact descriptor must contain path/sha256")
    path = safe_file(value["path"], base)
    require(sha(path) == value["sha256"], f"Artifact hash mismatch: {path.name}")
    return path


def verify_preparation():
    plan = safe_file("PREPARATION-PLAN-v2.json", EXPERIMENT)
    require(sha(plan) == PLAN_SHA, "Frozen preparation plan changed")
    preparation = read(plan)
    require(sha(safe_file("PREPARATION-PLAN.json", EXPERIMENT)) == preparation["supersedes_plan_sha256"] == PREVIOUS_PLAN_SHA, "Original preparation plan changed")
    require(preparation["numerical_output_cap_bytes_overall"] == OUTPUT_CAP and preparation["numerical_output_directories"] == list(OUTPUT_DIRECTORIES) and preparation["producer_peak_rss_cap_bytes"] == RSS_CAP, "Resource caps differ")
    binding = read(safe_file("SOURCE-INPUT-BINDING.json", EXPERIMENT))
    require(binding["source_commit"] == preparation["source_commit"] == SOURCE, "Wrong source commit")
    require(len(preparation["source_hashes"]) == 40, "Expected forty frozen source files")
    expected = {"source/"+name: digest for name, digest in preparation["source_hashes"].items()}
    expected.update(preparation["inputs_sha256"])
    require(binding["files"] == expected, "Source/input bindings conflict")
    inventory = {str(p.relative_to(EXPERIMENT / "source")) for p in (EXPERIMENT / "source").rglob("*") if p.is_file()}
    require(inventory == set(preparation["source_hashes"]), "Frozen source inventory changed")
    for name, digest in expected.items():
        require(sha(safe_file(name, EXPERIMENT)) == digest, f"Frozen source/input changed: {name}")
    producer_path = safe_file("PRODUCER-BINDING-v2.json", EXPERIMENT)
    require(sha(producer_path) == PRODUCER_BINDING_SHA, "Producer binding changed")
    producer = read(producer_path)
    require(producer["source_commit"] == SOURCE and producer["plan_sha256"] == PLAN_SHA and producer["source_input_binding_sha256"] == sha(EXPERIMENT / "SOURCE-INPUT-BINDING.json"), "Producer source/input disagreement")
    require(producer["build_confirmation_exit_code"] == 0 and producer["private_real_target_no_symlinks"] is True, "Producer preparation not confirmed")
    for name, digest in producer["files"].items():
        require(sha(safe_file(name, EXPERIMENT)) == digest, f"Producer instrumentation changed: {name}")
    require(sha(safe_file(producer["driver_path"], EXPERIMENT)) == producer["driver_sha256"], "Actual driver changed")
    for case in REVISIONS:
        preflight = producer["preflight"][case]
        actual = safe_file(f"preflight-{case}/mesh-metadata.json", EXPERIMENT)
        obj = safe_file(f"preflight-{case}/rest.obj", EXPERIMENT)
        require(sha(actual) == preflight["metadata_sha256"] and sha(obj) == preflight["rest_obj_sha256"], "Geometry preflight changed")
        old = read(GOLD / f"runs/{case}/rest-metadata.json")
        old.pop("masses_kg")
        require(same(read(actual), old), "Exact current/old geometry preflight differs")
        require(obj.read_bytes() == (GOLD / f"prepared/{case}/rest.obj").read_bytes(), "Current/old rest OBJ bytes differ")
    return {"plan_sha256": PLAN_SHA, "source_commit": SOURCE, "source_files": 40,
            "source_input_binding_sha256": sha(EXPERIMENT / "SOURCE-INPUT-BINDING.json"), "producer_binding_sha256": PRODUCER_BINDING_SHA,
            "driver_sha256": producer["driver_sha256"], "all_hashes_exact": True}


def verify_frozen():
    freeze = read(ROOT / "FREEZE.json")
    require(freeze["status"] == "frozen_preparation_no_garment_execution", "Evaluator freeze absent")
    for name, digest in freeze["files"].items():
        require(sha(safe_file(name, ROOT)) == digest, f"Evaluator changed after freeze: {name}")
    verify_readonly_inputs()
    return freeze


def verify_readonly_inputs():
    binding = read(ROOT / "READONLY-INPUT-BINDING.json")
    for name, digest in binding["files"].items():
        require(sha(safe_file(name, REPO)) == digest, f"Prior independent input/evidence changed: {name}")
    return {"files": len(binding["files"]), "all_hashes_exact": True}


def authorization(path):
    path = safe_file(path, EXPERIMENT)
    auth = read(path)
    require(type(auth["schema"]) is int and auth["schema"] == 1 and auth["status"] == "authorized", "Root execution authorization absent")
    require(auth["source_commit"] == SOURCE and auth["cases"] == ["base", "height"] and type(auth["attempts_per_case"]) is int and auth["attempts_per_case"] == 1, "Wrong root execution scope")
    require(descriptor(auth["runner"], REPO) == EXPERIMENT / "runner.py", "Wrong actual execution wrapper")
    require(descriptor(auth["plan"], REPO) == EXPERIMENT / "PREPARATION-PLAN-v2.json", "Wrong plan")
    require(descriptor(auth["source_input_binding"], REPO) == EXPERIMENT / "SOURCE-INPUT-BINDING.json", "Wrong source/input binding")
    producer_binding = descriptor(auth["producer_binding"], REPO)
    require(producer_binding == EXPERIMENT / "PRODUCER-BINDING-v2.json" and sha(producer_binding) == PRODUCER_BINDING_SHA, "Wrong actual producer binding")
    require(descriptor(auth["evaluator_freeze"], REPO) == ROOT / "FREEZE.json", "Wrong independent evaluator")
    verify_frozen()
    verify_preparation()
    return auth, read(producer_binding)


def settings(frame):
    return {"dt": 1/60, "substeps": 8, "iterations": 32, "seam_closure_speed": 1.,
            "gravity": [0., 0., 0.] if frame <= 60 else [0., -9.81, 0.]}


def point(positions, endpoint):
    a, b = (positions[v] for v in endpoint["vertices"])
    return [(1-endpoint["weight"])*x+endpoint["weight"]*y for x, y in zip(a, b)]


def initial_targets(meta):
    return [metrics.norm(metrics.sub(point(meta["rest_positions_m"], s["a"]), point(meta["rest_positions_m"], s["b"]))) for s in meta["stitches"]]


def target_step(targets):
    for _ in range(8):
        targets = [max(0., t-1.*((1/60)/8)) for t in targets]
    return targets


def check_metadata(meta, case):
    require(case in REVISIONS, "Unknown physical case")
    bindings = read(ROOT / "REST-BINDING.json")
    old_path = safe_file(bindings[case]["path"], REPO)
    require(sha(old_path) == bindings[case]["sha256"], "Frozen old rest metadata changed")
    expected = read(old_path)
    require(same(meta, expected), "Exact actual rest metadata/project/UV/topology/material/boundary/stitch/mass identity failed")
    authored = read(EXPERIMENT / f"inputs/{case}.weftforge")["project"]
    serialized = copy.deepcopy(authored)
    for material in serialized["materials"]:
        material["color"] = [struct.unpack(">f", struct.pack(">f", x))[0] for x in material["color"]]
    require(same(meta["project"], serialized), "Unexpected authored-to-rest field conversion beyond known f32 color storage")
    require(meta["doc_revision"] == meta["project"]["revision"] == REVISIONS[case], "Wrong authoritative revision")
    require(meta["all_contact_enabled"] is True and meta["pins"] == [] and same(meta["ground_height_m"], 0.), "Wrong contact/pin/ground settings")
    require(not meta["project"]["mannequin"].get("ellipsoids", []), "Unexpected ellipsoid in bounded capsule-only input")
    expected_masses = metrics.masses(meta)
    error = max(abs(a-b) for a, b in zip(expected_masses, meta["masses_kg"]))
    require(error <= 1e-12 and all(m > 0 for m in expected_masses), "Independent physical mass check failed")
    return {"exact_old_rest_metadata": True, "exact_authoring_except_declared_f32_color_storage": True,
            "doc_revision": REVISIONS[case], "vertices": len(expected_masses), "triangles": len(meta["triangles"]),
            "max_independent_mass_error_kg": error, "total_independent_mass_kg": sum(expected_masses)}


def screen(strain, seam, contact, target, finite, uncertain, reported_uncertain=0):
    return {"finite": finite, "material_axis_strain_below_0_15": strain < .15,
            "sampled_seam_gap_below_0_005_m": seam < .005, "physical_contact_below_0_002_m": contact < .002,
            "completed_assembly_target_exact_zero": target == 0.,
            "independent_capsule_unknown_pairs_exact_zero": type(uncertain) is int and uncertain == 0,
            "reported_capsule_conservative_pairs_exact_zero": type(reported_uncertain) is int and reported_uncertain == 0}


def finite_vectors(value, count):
    return isinstance(value, list) and len(value) == count and all(isinstance(v, list) and len(v) == 3 and all(number(q) for q in v) for v in value)


def check_obj(meta, positions, path):
    actual = metrics.parse_obj(path)
    expected_groups = []
    for tri in meta["triangles"]:
        group = f"piece_{meta['piece_ids'][tri[0]]}"
        if not expected_groups or expected_groups[-1] != group:
            expected_groups.append(group)
    require(actual["g"] == expected_groups, "Unexpected or duplicate OBJ piece groups")
    return {**metrics.check_obj(meta, positions, path), "exact_piece_group_sequence": True,
            "sha256": sha(path), "comments": [line for line in path.read_text().splitlines() if line.lstrip().startswith("#")]}


def measure_frame(meta, state, frame, step, targets, expected_time, excluded, *, legacy=False, initial_intersections=None, initial_count_verified=True):
    issues = []
    def check(condition, message):
        if not condition:
            issues.append(message)
    def close(actual, expected, tolerance, message):
        check(number(actual) and number(expected) and abs(actual-expected) <= tolerance, message)
    check(type(state.get("global_frame")) is int and state["global_frame"] == frame, "state frame sequence mismatch")
    check(same(state.get("settings_last_step"), settings(frame)), "actual state settings mismatch")
    check(state.get("phase") == ("assembly" if frame <= 60 else "drape"), "state phase mismatch")
    expected_keys = {"global_frame", "settings_last_step", "phase", "diagnostics", "positions_m", "velocities_m_per_s", "case_wall_seconds" if legacy else "producer_wall_seconds"}
    check(set(state) == expected_keys, "unexpected state schema keys")
    if frame:
        check(step is not None and set(step) == {"global_frame", "settings", "measured_step_seconds", "diagnostics"}, "missing or invalid actual step record")
        if step:
            check(type(step.get("global_frame")) is int and step["global_frame"] == frame, "step frame sequence mismatch")
            check(same(step.get("settings"), settings(frame)), "actual step settings mismatch")
            check(same(step.get("diagnostics"), state.get("diagnostics")), "state/step raw diagnostic mismatch")
            check(number(step.get("measured_step_seconds")) and step["measured_step_seconds"] >= 0., "invalid measured step clock")
    x, v, diagnostics = state.get("positions_m"), state.get("velocities_m_per_s"), state.get("diagnostics")
    finite = finite_vectors(x, len(meta["piece_ids"])) and finite_vectors(v, len(meta["piece_ids"]))
    check(finite, "invalid/nonfinite actual positions or velocities")
    row = {"frame": frame, "raw_diagnostics": diagnostics, "issues": issues, "settings_time_target_agreement": False,
           "completed_fit_phase": max(targets, default=0.) == 0.}
    if not finite or not isinstance(diagnostics, dict):
        row.update(quality_gates={"finite": False}, quality_passed=False, exact_attribution_agreement=False,
                   required_diagnostic_verification={"state_measurement_available": False})
        return row
    if frame == 0:
        check(same(x, meta["rest_positions_m"]), "initial positions differ from actual rest metadata")
        check(all(q == 0. for vector in v for q in vector), "initial velocity is nonzero")
    target = max(targets, default=0.)
    time_ok = same(diagnostics.get("simulation_time"), expected_time)
    target_ok = same(diagnostics.get("seam_assembly_remaining"), target)
    check(time_ok, "exact repeated-dt simulation time recurrence mismatch")
    check(target_ok, "exact per-stitch/substep closure recurrence mismatch")
    check(diagnostics.get("finite") is True, "producer finite flag false")
    strain = metrics.measure(meta, x)
    seam_rows = [(metrics.norm(metrics.sub(point(x, s["a"]), point(x, s["b"]))), i, s["seam"]) for i, s in enumerate(meta["stitches"])]
    seam, stitch_index, seam_id = max(seam_rows, default=(0., None, None))
    analytic = metrics.analytic_contacts(meta, x)
    supported = metrics.supported_self_contact(meta, x, excluded)
    capsule = capsule_geometry.measure(meta, x)
    policy = capsule_geometry.conservative_policy_zero_certificate(meta, x)
    contact_lower = max(analytic["avatar_ground_penetration_m"], supported["supported_static_self_interpanel_penetration_m"], capsule["penetration_lower_m"])
    contact_upper = max(analytic["avatar_ground_penetration_m"], supported["supported_static_self_interpanel_penetration_m"], capsule["penetration_upper_m"])
    kinetic = sum(m*metrics.dot(velocity, velocity)/2 for m, velocity in zip(meta["masses_kg"], v))
    close(diagnostics.get("max_stretch_error"), strain["peak"]["absolute_strain"], 1e-12, "diagnostic material-axis magnitude mismatch")
    close(diagnostics.get("max_seam_error"), seam, 1e-12, "diagnostic weighted stitch gap mismatch")
    close(diagnostics.get("kinetic_energy"), kinetic, 1e-12*(1+abs(kinetic)), "diagnostic physical kinetic energy mismatch")
    reported = diagnostics.get("peak_strain")
    attribution = isinstance(reported, dict) and type(reported.get("triangle")) is int and reported["triangle"] == strain["peak"]["triangle_index"] and reported.get("axis") == strain["peak"]["axis"]
    check(attribution, "EXACT diagnostic peak triangle/axis attribution mismatch")
    if isinstance(reported, dict) and type(reported.get("triangle")) is int and 0 <= reported["triangle"] < len(meta["triangles"]) and reported.get("axis") in ("warp", "weft"):
        nominated = metrics.signed_triangle_axes(meta, x, reported["triangle"])[reported["axis"]]
        close(reported.get("signed_strain"), nominated, 1e-12, "diagnostic nominated triangle/axis signed strain mismatch")
        close(abs(reported["signed_strain"]), strain["peak"]["absolute_strain"], 1e-12, "diagnostic nominated peak magnitude mismatch")
    else:
        check(False, "invalid peak-strain diagnostic attribution")
    if not legacy:
        diagnostic_contact = diagnostics.get("max_contact_error")
        check(number(diagnostic_contact) and contact_lower-1e-10 <= diagnostic_contact <= contact_upper+1e-10, "diagnostic full static physical-contact residual mismatch")
        reported_uncertainty = diagnostics.get("capsule_surface_uncertain_pairs")
        check(type(reported_uncertainty) is int and reported_uncertainty >= 0, "invalid diagnostic conservative capsule-pair count")
        policy_verified = policy["zero_count_proved"] and type(reported_uncertainty) is int and reported_uncertainty == 0
        if policy["zero_count_proved"] and type(reported_uncertainty) is int:
            check(reported_uncertainty == 0, "diagnostic conservative capsule-pair count contradicts all-pair well-conditioned zero proof")
    else:
        reported_uncertainty = 0
        policy_verified = True
    raw_initial = diagnostics.get("initial_intersections")
    raw_initial_valid = type(raw_initial) is int and raw_initial >= 0
    check(raw_initial_valid, "invalid diagnostic initial-intersection count")
    for key in ("contacts",):
        check(type(diagnostics.get(key)) is int and diagnostics[key] >= 0, f"invalid diagnostic {key} count")
    initial_verified = False
    if frame == 0:
        vertex_supported = analytic["intersection_pairs_depth_over_1e_minus8"] + supported["intersection_pairs_depth_over_1e_minus8"]
        lower_initial = vertex_supported+capsule["counts"]["intersection_pairs_depth_over_1e_minus8_lower"]
        upper_initial = vertex_supported+capsule["counts"]["intersection_pairs_depth_over_1e_minus8_upper"]
        exact_policy = policy["zero_count_proved"]
        initial_verified = exact_policy and raw_initial_valid and lower_initial == upper_initial == raw_initial and capsule["uncertain_pairs"] == 0
        if exact_policy:
            check(not raw_initial_valid or lower_initial <= raw_initial <= upper_initial, "independent initial vertex/self/surface intersection count mismatch")
            row["initial_intersection_count_status"] = "verified" if initial_verified else ("unknown_ambiguous_threshold_or_geometry" if raw_initial_valid and lower_initial <= raw_initial <= upper_initial else "failed")
        else:
            row["initial_intersection_count_status"] = "unverified_conservative_policy_may_add_intersections" if raw_initial_valid else "failed"
        row["initial_intersections_expected_producer_count_range"] = [lower_initial, upper_initial] if exact_policy else None
        row["initial_intersections_independent_range"] = {"lower": lower_initial, "upper": upper_initial,
                                                        "vertex_supported": vertex_supported, "capsule_threshold_ambiguous_pairs": capsule["counts"]["intersection_threshold_ambiguous_pairs"]}
    else:
        stable_initial = raw_initial_valid and type(initial_intersections) is int and raw_initial == initial_intersections
        check(stable_initial, "retained initial-intersection count changed or original count unavailable")
        initial_verified = stable_initial and initial_count_verified
        row["initial_intersection_count_status"] = "verified" if initial_verified else ("unknown_original_count_unverified" if stable_initial else "failed")
    row["initial_intersections_raw"] = raw_initial
    row["reported_conservative_capsule_pairs"] = None if legacy else reported_uncertainty
    row["independent_unknown_capsule_pairs"] = capsule["uncertain_pairs"]
    row["product_uncertainty_policy_count_agreement"] = "not_measured_in_old_diagnostics" if legacy else ("expected_zero_proved_for_all_pairs" if policy_verified else ("failed_expected_zero" if policy["zero_count_proved"] else "unverified_geometry_policy"))
    row["product_uncertainty_policy_zero_certificate"] = policy
    row["required_diagnostic_verification"] = {"product_uncertainty_policy_count_agreement": policy_verified,
                                               "initial_intersection_count_agreement": initial_verified}
    gates = screen(strain["peak"]["absolute_strain"], seam, contact_upper, target, finite and diagnostics.get("finite") is True, capsule["uncertain_pairs"], reported_uncertainty)
    row.update(independent_material_axis=strain, independent_seam={"gap_m": seam, "stitch_index": stitch_index, "seam_id": seam_id},
               independent_analytic_vertex_ground=analytic, independent_supported_static_self_contact=supported,
               independent_full_static_capsule=capsule, physical_contact_lower_m=contact_lower, physical_contact_upper_m=contact_upper,
               independent_kinetic_energy_J=kinetic, expected_time_s=expected_time, expected_max_stitch_target_m=target,
               exact_attribution_agreement=attribution, settings_time_target_agreement=time_ok and target_ok and same(state.get("settings_last_step"), settings(frame)),
               quality_gates=gates, quality_passed=all(gates.values()), diagnostic_agreement=not issues,
               rms_speed_m_per_s=math.sqrt(2*kinetic/sum(meta["masses_kg"])), max_speed_m_per_s=max(map(metrics.norm, v)))
    return row


def summarize(result):
    rows = result["rows"]
    names = sorted({key for row in rows for key in row["quality_gates"]})
    result["failed_frames_by_screen"] = {name: [row["frame"] for row in rows if not row["quality_gates"].get(name, False)] for name in names}
    result["diagnostic_disagreement_frames"] = [row["frame"] for row in rows if row["issues"]]
    result["exact_attribution_failed_frames"] = [row["frame"] for row in rows if not row["exact_attribution_agreement"]]
    result["blocked_diagnostic_verification_frames"] = [row["frame"] for row in rows if not all(row.get("required_diagnostic_verification", {}).values())]
    result["first_exact_zero_target_frame"] = next((row["frame"] for row in rows if row["completed_fit_phase"]), None)
    result["all_frame_quality_passed"] = len(rows) == 181 and all(row["quality_passed"] for row in rows)
    completed = [row for row in rows if row["completed_fit_phase"]]
    result["completed_fit_all_frame_quality_passed"] = len(rows) == 181 and bool(completed) and all(row["quality_passed"] for row in completed)
    result["final_frame_quality_passed"] = len(rows) == 181 and rows[-1]["quality_passed"]
    physical = ("finite", "material_axis_strain_below_0_15", "physical_contact_below_0_002_m", "independent_capsule_unknown_pairs_exact_zero", "reported_capsule_conservative_pairs_exact_zero")
    result["trajectory_physical_screens_passed"] = len(rows) == 181 and all(all(row["quality_gates"].get(key, False) for key in physical) for row in rows)
    result["assembly_targets_zero_at_60_and_180"] = len(rows) == 181 and all(rows[frame]["quality_gates"].get("completed_assembly_target_exact_zero", False) for frame in (60, 180))
    result["initial_arrangement_unclosed_seams_are_completed_fit_failure"] = False
    result["strict_delivery_passed"] = result.get("status") == "analyzed_complete" and result["trajectory_physical_screens_passed"] and result["assembly_targets_zero_at_60_and_180"] and result["completed_fit_all_frame_quality_passed"] and not result["diagnostic_disagreement_frames"] and not result["blocked_diagnostic_verification_frames"] and not result.get("errors")
    result["quality_status"] = "passed" if result["strict_delivery_passed"] else "failed"


def analyze_saved(case, run, *, legacy=False, deadline=None):
    run = Path(run)
    started = time.monotonic()
    result = {"kind": "Independent original Harbor saved-state analysis", "case": case, "status": "incomplete_or_failed",
              "legacy_saved_gold_only": legacy, "new_garment_execution": False, "rows": [], "errors": [], "scope": SCOPE, "historical_failures": HISTORY}
    try:
        meta = read(safe_file("rest-metadata.json", run))
        result["rest_check"] = check_metadata(meta, case)
        excluded = metrics.exclusion(meta)
        targets, expected_time, previous_wall, initial_intersections, initial_count_verified = initial_targets(meta), 0., -1., None, False
        with safe_file("trajectory.jsonl", run).open() as states, safe_file("steps.jsonl", run).open() as steps:
            for frame, line in enumerate(states):
                require(frame <= 180, "Extra retained state after frame180")
                if deadline is not None and time.monotonic() >= deadline:
                    raise TimeoutError("Whole-case420second cap reached; analysis ended without continuation")
                if not legacy:
                    require(numerical_output_bytes() <= OUTPUT_CAP, "Numerical-output256MiB cap reached; analysis ended")
                state = strict_loads(line)
                step = None
                if frame:
                    step = strict_loads(next(steps))
                    targets, expected_time = target_step(targets), expected_time+1/60
                row = measure_frame(meta, state, frame, step, targets, expected_time, excluded, legacy=legacy, initial_intersections=initial_intersections, initial_count_verified=initial_count_verified)
                if frame == 0 and isinstance(state.get("diagnostics"), dict) and type(state["diagnostics"].get("initial_intersections")) is int:
                    initial_intersections = state["diagnostics"]["initial_intersections"]
                    initial_count_verified = row.get("required_diagnostic_verification", {}).get("initial_intersection_count_agreement", False)
                wall = state.get("case_wall_seconds" if legacy else "producer_wall_seconds")
                if not number(wall) or wall < previous_wall:
                    row["issues"].append("producer clock invalid or decreased")
                else:
                    previous_wall = wall
                if frame in (0, 60, 180):
                    path = GOLD / f"prepared/{case}/rest.obj" if legacy and frame == 0 else run / f"stage-{frame:03}.obj"
                    try:
                        row["obj_check"] = check_obj(meta, state["positions_m"], safe_file(path, GOLD if legacy else run))
                    except Exception as error:
                        row["issues"].append(f"OBJ validation failed: {type(error).__name__}: {error}")
                result["rows"].append(row)
            require(not steps.read().strip(), "Extra step records without retained states")
        failed_path = run / "failed-step.json"
        if failed_path.exists():
            failed = read(safe_file(failed_path, run))
            result["producer_failed_step"] = failed
            require(failed["attempted_frame"] == len(result["rows"]) and failed["last_retained_frame"] == len(result["rows"])-1, "Failed-step frame/accepted-prefix mismatch")
            require(same(failed["settings"], settings(failed["attempted_frame"])), "Failed-step attempted settings mismatch")
            retained = dict(failed["retained_state"])
            accepted = dict(state)
            retained.pop("producer_wall_seconds", None)
            accepted.pop("producer_wall_seconds", None)
            require(same(retained, accepted), "Failed step changed last accepted state/settings/diagnostics")
            result["failed_step_retained_state_exact"] = True
        require(len(result["rows"]) == 181, "Incomplete retained trajectory")
        require(not failed_path.exists(), "Both failed-step and full completion are present")
        complete = read(safe_file("completed-case.json", run))
        require(type(complete["completed_frames"]) is int and complete["completed_frames"] == 180 and same(complete["final_diagnostics"], result["rows"][-1]["raw_diagnostics"]), "Completion receipt/state mismatch")
        if not legacy:
            raw = complete["final_diagnostics"]
            raw_gates = screen(raw["max_stretch_error"], raw["max_seam_error"], raw["max_contact_error"], raw["seam_assembly_remaining"], raw["finite"], raw["capsule_surface_uncertain_pairs"], raw["capsule_surface_uncertain_pairs"])
            require(complete["final_quality_passed"] is all(raw_gates.values()), "Producer final-only quality flag mismatch")
        result["producer_completion"] = complete
        result["status"] = "analyzed_complete"
    except Exception as error:
        result["errors"].append(f"{type(error).__name__}: {error}")
    result["analysis_seconds"] = time.monotonic()-started
    summarize(result)
    return result


def receipt_check(receipt, case, auth_path, producer_binding, *, final=False):
    require(type(receipt["schema"]) is int and receipt["schema"] == 1 and receipt["case"] == case and type(receipt["attempt"]) is int and receipt["attempt"] == 1, "Wrong case receipt scope")
    require(receipt["source_commit"] == SOURCE, "Wrong receipt source")
    require(receipt["input_sha256"] == sha(EXPERIMENT / f"inputs/{case}.weftforge"), "Wrong receipt input")
    require(receipt["driver_sha256"] == producer_binding["driver_sha256"], "Wrong receipt driver")
    require(receipt["evaluator_freeze_sha256"] == sha(ROOT / "FREEZE.json"), "Wrong receipt evaluator")
    require(receipt["authorization_sha256"] == sha(auth_path), "Wrong receipt authorization")
    start, end = receipt["start_monotonic_seconds"], receipt["producer_end_monotonic_seconds"]
    require(number(start) and number(end) and 0 <= start <= end <= time.monotonic(), "Invalid actual process clock")
    require(type(receipt["producer_peak_rss_bytes"]) is int and receipt["producer_peak_rss_bytes"] > 0, "Producer RSS observation missing")
    require(type(receipt["numerical_output_bytes"]) is int and receipt["numerical_output_bytes"] >= 0, "Numerical-output volume observation missing")
    require(type(receipt["producer_exit_code"]) is int and receipt["status"] in ("producer_complete", "complete", "producer_failed", "producer_stopped", "failed"), "Invalid producer terminal status")
    producer_completed = receipt["producer_exit_code"] == 0 and receipt["status"] in ("producer_complete", "complete")
    run = EXPERIMENT / "runs" / case
    if producer_completed:
        require({"rest-metadata.json", "trajectory.jsonl", "steps.jsonl", "stage-000.obj", "stage-060.obj", "stage-180.obj", "completed-case.json"} <= set(receipt["artifacts"]), "Incomplete producer artifact manifest")
    for name, digest in receipt["artifacts"].items():
        require(sha(safe_file(name, run)) == digest, f"Receipt artifact changed: {name}")
    if final:
        last = receipt["final_monotonic_seconds"]
        require(receipt["status"] in ("complete", "failed") and number(last) and end <= last <= time.monotonic(), "Missing final inspection clock")
        require(number(receipt["whole_case_elapsed_seconds"]) and abs((last-start)-receipt["whole_case_elapsed_seconds"]) <= 1e-9, "Whole-case receipt clock mismatch")
        analysis = read(descriptor(receipt["analysis"], REPO))
        require(analysis["case"] == case and analysis["source_commit"] == SOURCE and analysis["authorization_sha256"] == sha(auth_path), "Final analysis binding mismatch")
        require(analysis["producer_artifacts"] == receipt["artifacts"] and same(analysis["case_start_monotonic_seconds"], start), "Final analysis/state/clock binding mismatch")
        require(analysis["analysis_completed_monotonic_seconds"] <= last, "Final receipt precedes analysis")
        descriptor(receipt["inspection"], REPO)
    return {"binding_exact": True, "producer_completed": producer_completed,
            "producer_rss_within_cap": receipt["producer_peak_rss_bytes"] <= RSS_CAP,
            "numerical_output_within_cap": max(receipt["numerical_output_bytes"], numerical_output_bytes()) <= OUTPUT_CAP,
            "producer_seconds": end-start, "producer_within_case_cap": end-start <= CAP_SECONDS,
            "whole_case_checked": final, "whole_case_within_cap": receipt["whole_case_elapsed_seconds"] <= CAP_SECONDS if final else None}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("command", choices=["analyze", "validate-receipt"])
    parser.add_argument("--case", choices=list(REVISIONS), required=True)
    parser.add_argument("--authorization", type=Path, required=True)
    parser.add_argument("--receipt", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    report = {"status": "refused", "new_garment_execution": False, "scope": SCOPE, "historical_failures": HISTORY}
    try:
        auth_path = safe_file(args.authorization, EXPERIMENT)
        auth, producer_binding = authorization(auth_path)
        receipt_path = safe_file(args.receipt, EXPERIMENT)
        receipt = read(receipt_path)
        receipt_hash = sha(receipt_path)
        bound = receipt_check(receipt, args.case, auth_path, producer_binding, final=args.command == "validate-receipt")
        if args.command == "validate-receipt":
            report.update(status="receipt_verified", receipt_check=bound,
                          execution_receipt_passed=all(bound[key] for key in ("producer_completed", "producer_rss_within_cap", "numerical_output_within_cap", "producer_within_case_cap", "whole_case_within_cap")))
        else:
            report = analyze_saved(args.case, EXPERIMENT / "runs" / args.case, deadline=receipt["start_monotonic_seconds"]+CAP_SECONDS)
            report["producer_receipt_check"] = bound
            report["source_commit"] = SOURCE
            report["producer_artifacts"] = receipt["artifacts"]
            report["case_start_monotonic_seconds"] = receipt["start_monotonic_seconds"]
            for key in ("producer_completed", "producer_rss_within_cap", "numerical_output_within_cap", "producer_within_case_cap"):
                if not bound[key]:
                    report["errors"].append(f"Producer receipt failed: {key}")
            report["whole_case_seconds_through_analysis"] = time.monotonic()-receipt["start_monotonic_seconds"]
            if report["whole_case_seconds_through_analysis"] > CAP_SECONDS:
                report["status"] = "whole_case_cap_failed"
                report["errors"].append("Whole-case cap exceeded through analysis; no continuation authorized")
            receipt_check(receipt, args.case, auth_path, producer_binding)
            summarize(report)
        require(sha(receipt_path) == receipt_hash, "Receipt changed during analysis")
        authorization(auth_path)
        report["authorization_sha256"] = sha(auth_path)
        report["receipt_sha256"] = receipt_hash
        report["analysis_completed_monotonic_seconds"] = time.monotonic()
        if args.command == "analyze":
            report["whole_case_seconds_through_analysis"] = time.monotonic()-receipt["start_monotonic_seconds"]
            require(report["whole_case_seconds_through_analysis"] <= CAP_SECONDS, "Whole-case cap exceeded through final input/hash inspection")
    except Exception as error:
        report["status"] = "refused_or_failed"
        report["strict_delivery_passed"] = False
        report["quality_status"] = "failed"
        report.setdefault("errors", []).append(f"{type(error).__name__}: {error}")
    write_report(args.output, report)
    print(json.dumps({key: report.get(key) for key in ("case", "status", "strict_delivery_passed", "completed_fit_all_frame_quality_passed", "errors")}))
    if report["status"] == "receipt_verified":
        return 0 if report["execution_receipt_passed"] else 2
    if report["status"] == "analyzed_complete":
        return 0 if report["strict_delivery_passed"] else 2
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
