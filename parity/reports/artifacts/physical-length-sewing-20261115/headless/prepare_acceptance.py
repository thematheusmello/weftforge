#!/usr/bin/env python3
"""Author the original input and prospective expectations, without running WeftForge."""
import hashlib
import json
import math
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parent
BASELINE = "e8dede8f949dd8ebbdfead4f17d86c5eb09fefc2"
CONTRACT_SHA = "4016013d9dda6bf4f910e52224586925314c156b9489d3c6d4a0599411fa18b2"


def dump(name, value):
    (ROOT / name).write_text(json.dumps(value, indent=2, allow_nan=False) + "\n")


def primitive(t):
    return 0.1 * (t * math.sqrt(1 + t * t) + math.asinh(t))


def collinear(t):
    return 0.03 * t * t + 0.17 * t * t * t


def invert(fn, target):
    lo, hi = 0.0, 1.0
    for _ in range(64):
        mid = (lo + hi) / 2
        if fn(mid) < target:
            lo = mid
        else:
            hi = mid
    return (lo + hi) / 2


def side(curve, start, direction, length):
    return dict(curve=curve, start_t=start, direction=direction, length_m=length)


def command(a=None, b=None, name="Original physical pair"):
    return {"type": "add_sewing_by_length", "data": {
        "name": name,
        "a": a or side(6, 0.0, "forward", 0.1),
        "b": b or side(15, 0.75, "reverse", 0.1),
    }}


def expected(side_spec, collinear_b=False):
    curve = side_spec["curve"]
    start = side_spec["start_t"]
    sign = 1 if side_spec["direction"] == "forward" else -1
    if curve == 6:
        fn = primitive
        point = lambda t: [0.2 * t, 0.1 * t * t]
    elif curve == 15:
        fn = collinear if collinear_b else lambda t: 0.2 * t
        point = lambda t: [0.4 + fn(t), 0.0]
    elif curve == 8:
        fn = lambda t: 0.2 * t
        point = lambda t: [0.2 * (1 - t), 0.3]
    else:
        raise AssertionError("Unplanned original curve")
    target = fn(start) + sign * side_spec["length_m"]
    exact = None
    # Straight full-remaining requests are chosen from the original f64 coordinates.
    # Their exact endpoint expectations do not depend on a production metric query.
    if curve == 8 and side_spec["length_m"] == 0.2:
        exact = 1.0 if sign > 0 else 0.0
    if curve == 15 and not collinear_b and side_spec["length_m"] == 0.6 - 0.4:
        exact = 1.0 if sign > 0 else 0.0
    end = exact if exact is not None else invert(fn, target)
    fractions = []
    for f in [0.0, 0.25, 0.5, 0.75, 1.0]:
        t = start if f == 0 else end if f == 1 else invert(fn, fn(start) + sign * side_spec["length_m"] * f)
        fractions.append(dict(fraction=f, t=t, point_m=point(t)))
    return dict(curve=curve, t0=start, t1=end, length_m=side_spec["length_m"], exact_end=exact,
                samples=fractions)


def main():
    fixture = {"format": "weftforge", "version": 3, "project": {
        "revision": 41,
        "points": [
            {"id": i, "position": p} for i, p in [
                (2, [0.0, 0.0]), (3, [0.2, 0.1]), (4, [0.2, 0.3]), (5, [0.0, 0.3]),
                (11, [0.4, 0.0]), (12, [0.6, 0.0]), (13, [0.6, 0.3]), (14, [0.4, 0.3]),
            ]],
        "curves": [
            {"id": i, "start": a, "end": b, "controls": c} for i, a, b, c in [
                (6, 2, 3, [[0.2 / 3, 0.0], [0.4 / 3, 0.1 / 3]]),
                (7, 3, 4, None), (8, 4, 5, None), (9, 5, 2, None),
                (15, 11, 12, None), (16, 12, 13, None), (17, 13, 14, None), (18, 14, 11, None),
            ]],
        "pieces": [
            {"id": i, "name": n, "boundary": b,
             "placement": {"translation": [0.0, 0.0, 0.0], "rotation": [0.0, 0.0, 0.0]},
             "material": 1, "grain_angle": 0.0}
            for i, n, b in [(10, "Original analytic panel", [6, 7, 8, 9]),
                            (19, "Original straight panel", [15, 16, 17, 18])]],
        "materials": [{"id": 1, "name": "Original uncalibrated material", "areal_density": 0.18,
                       "warp_stiffness": 400.0, "weft_stiffness": 400.0, "shear_stiffness": 100.0,
                       "bending_stiffness": 0.003, "damping": 0.2, "friction": 0.4,
                       "collision_thickness": 0.003, "color": [0.25, 0.5, 0.75]}],
        "seams": [], "sewing_groups": [], "mannequin": {"capsules": []},
        "resolution": 0.07, "next_id": 20,
    }}
    dump("fixture.weftforge", fixture)
    cases = [
        ("initial_asymmetric", command(), {}),
        ("interior_reverse", command(side(6, 0.8, "reverse", 0.1), side(15, 0.1, "forward", 0.1)), {}),
        ("interior_forward", command(side(6, 0.2, "forward", 0.1), side(15, 0.9, "reverse", 0.1)), {}),
        ("near_full_forward", command(side(6, 0.0, "forward", primitive(1.0) - 1e-6),
                                      side(15, 0.0, "forward", 0.19)), {}),
        ("near_full_reverse", command(side(6, 1.0, "reverse", primitive(1.0) - 1e-6),
                                      side(15, 1.0, "reverse", 0.19)), {}),
        ("exact_forward", command(side(8, 0.0, "forward", 0.2),
                                 side(15, 0.0, "forward", 0.6 - 0.4)), {}),
        ("exact_reverse", command(side(8, 1.0, "reverse", 0.2),
                                 side(15, 1.0, "reverse", 0.6 - 0.4)), {}),
        ("disjoint_same_curve", command(side(6, 0.0, "forward", 0.04),
                                       side(6, 1.0, "reverse", 0.04)), {}),
        ("unequal_existing_proportional", command(b=side(15, 0.9, "reverse", 0.15)), {}),
        ("nonuniform_collinear", command(b=side(15, 0.0, "forward", 0.1)), {"collinear_b": True}),
        ("nonuniform_collinear_reverse", command(b=side(15, 1.0, "reverse", 0.1)), {"collinear_b": True}),
        ("legacy_nonlinear_preserved", command(), {"legacy_seam": True}),
    ]
    positives = [dict(id=i, command=c, variant=v,
                      expected={k: expected(c["data"][k], v.get("collinear_b", False)) for k in ["a", "b"]})
                 for i, c, v in cases]
    negatives = []

    def reject(i, c, variant=None, same_file=False, category="validation"):
        negatives.append(dict(id=i, commands=c if isinstance(c, list) else [c],
                              variant=variant or {}, same_file=same_file, category=category))

    for i, field, value in [
        ("negative_start", "start_t", -1e-12), ("past_end_start", "start_t", 1.000000000001),
        ("zero_length", "length_m", 0.0), ("negative_length", "length_m", -0.1),
        ("enormous_length", "length_m", 1e308), ("missing_curve", "curve", 999),
        ("collapsed_parameter_span", "length_m", 1e-12),
        ("unknown_direction", "direction", "sideways"), ("numeric_direction", "direction", -1),
        ("null_start", "start_t", None),
    ]:
        c = command()
        c["data"]["a"][field] = value
        reject(i, c, same_file=i == "missing_curve", category="schema" if i in ["unknown_direction", "numeric_direction", "null_start"] else "validation")
    reject("forward_no_room", command(side(6, 1.0, "forward", 0.1)))
    reject("reverse_no_room", command(side(6, 0.0, "reverse", 0.1)))
    reject("overlong_small_excess", command(b=side(15, 0.5, "forward", 0.100000000001)), same_file=True)
    reject("same_curve_overlap", command(side(6, 0.0, "forward", 0.1), side(6, 0.2, "forward", 0.1)))
    reject("invalid_name", command(name=" \t "))
    reject("id_overflow", command(), {"next_id": 18446744073709551615})
    reject("revision_overflow", command(), {"revision": 18446744073709551615})
    c = command()
    c["data"]["a"]["unexpected"] = True
    reject("unknown_side_field", c, category="schema")
    c = command()
    c["data"]["unexpected"] = True
    reject("unknown_data_field", c, category="schema")
    c = command()
    del c["data"]["b"]["direction"]
    reject("missing_direction", c, category="schema")
    c = command()
    c["unexpected"] = True
    reject("unknown_command_field", c, category="schema")
    # A successful earlier command in the array must not leak through a later failure.
    reject("transaction_second_side_failure", [
        {"type": "set_resolution", "data": {"metres": 0.035}},
        command(b=side(15, 1.0, "forward", 0.1))], same_file=True)
    # This checks existing group metric validation after physical creation, not an
    # initial AddSewingByLength metric failure or work-budget saturation.
    good = positives[0]["expected"]
    group = {"id": 20, "name": "Original metric precision probe",
             "a": [{k: good["a"][k] for k in ["curve", "t0", "t1"]}],
             "b": [{k: good["b"][k] for k in ["curve", "t0", "t1"]}],
             "correspondence": [[0.0, 0.0], [1e-18, 0.5], [1.0, 1.0]]}
    reject("composed_group_metric_precision", [command(), {"type": "update_sewing_group", "data": {"group": group}}],
           same_file=True, category="metric_precision_composition")
    raw_base = json.dumps([command()], separators=(",", ":"))
    raw_cases = [
        dict(id="nan_start_token", raw=raw_base.replace('"start_t":0.0', '"start_t":NaN', 1)),
        dict(id="infinite_length_token", raw=raw_base.replace('"length_m":0.1', '"length_m":Infinity', 1)),
        dict(id="overflowing_json_number", raw=raw_base.replace('"length_m":0.1', '"length_m":1e400', 1)),
        dict(id="duplicate_start_field", raw=raw_base.replace('"start_t":0.0', '"start_t":0.0,"start_t":0.2', 1)),
        dict(id="duplicate_command_type", raw=raw_base.replace('"type":"add_sewing_by_length"',
             '"type":"add_sewing_by_length","type":"add_sewing_by_length"', 1)),
    ]
    criteria = dict(
        schema_version=1, status="prospective_frozen_engineering_acceptance",
        contract_sha256=CONTRACT_SHA, immutable_source_commit=BASELINE,
        metric_tolerance_m=1e-8, oracle_crosscheck_tolerance_m=2e-12,
        straight_t_tolerance=2e-14,
        oracle_methods=["analytic primitive and 64-step bisection", "independent derivative composite Simpson at 2048/4096 steps and 56-step bisection"],
        primitive="F(t)=0.1*(t*sqrt(1+t*t)+asinh(t)); C(t)=(0.2t,0.1t^2)",
        full_curved_length_m=primitive(1.0), fractions=[0.0, 0.25, 0.5, 0.75, 1.0],
        positive_cases=positives, negative_cases=negatives, raw_schema_cases=raw_cases,
        channels=["cli_execute", "mcp_2026-07-28", "mcp_2025-11-25"],
        coarse_fine_resolutions_m=[0.09, 0.035],
        writer_failure_cases=["invalid_existing_destination", "destination_directory", "destination_symlink", "recovery_directory", "absent_parent", "invalid_input"],
        separate_required_gates=[
            "Native first-side transient; second-side one command/transaction; Undo/Redo IDs and fields; save/new/reopen.",
            "Real pointer workflow, exact endpoint picking/ties, controls, Tab/OK, labels/guides, keyboard accessibility and modal shielding.",
            "Cancel/close/Escape/tool switch/reset/history/load/stale revision and raw text/caret/inspector preservation.",
            "Source/unit checks: initial AddSewingByLength metric-failure injection, cancellation and work-budget exhaustion; atomic-writer pre-replace interruption.",
            "All four required local Rust gates and clean tracked evidence registry/map validation.",
        ],
        claim_limits=[
            "Independent authored engineering fixture; no reference metric, projection, cancellation, easing, solver or full sewing parity.",
            "CLI/MCP are stateless files; this evaluator cannot observe live Editor history, GUI drafts, caches or solver state.",
            "No candidate output generates a numerical expectation; candidate endpoints are reused only as input for the existing map-knot probe after independent endpoint validation.",
            "Exact endpoint cases use independent straight arithmetic; analytic full-curved length may differ from a computed metric by ulps and is covered by near-full tests instead.",
            "Metric precision composition is separately labelled; no large work saturation workload or disk-full claim.",
        ],
    )
    dump("CRITERIA.json", criteria)
    paths = [
        "spec/features/SEW-002-physical-length.md", "spec/features/SEW-002-ordered-paths.md",
        "spec/features/SEW-002-paired-runtime-20261114.md", "spec/features/SEW-002-reverse-runtime-20261114.md",
        "spec/features/SEW-002-runtime-20261114.md",
        "spec/observations/REF-SEW-002-FREE-PARTIAL-001.json", "spec/observations/REF-SEW-002-PAIR-002.json",
        "spec/observations/REF-SEW-002-REVERSE-003.json",
        "crates/weftforge-core/src/document.rs", "crates/weftforge-core/src/persistence.rs",
        "crates/weftforge-core/src/sewing.rs", "crates/weftforge-core/tests/sewing_groups.rs",
        "apps/weftforge-cli/src/main.rs", "apps/weftforge-cli/src/mcp.rs",
    ]
    sources = []
    for path in paths:
        raw = subprocess.check_output(["git", "show", f"{BASELINE}:{path}"], cwd=ROOT)
        sources.append(dict(commit=BASELINE, path=path, sha256=hashlib.sha256(raw).hexdigest()))
    assert sources[0]["sha256"] == CONTRACT_SHA
    dump("SOURCE-INPUTS.json", dict(
        schema_version=1, sources=sources,
        use="Public reviewed behavior records bound observations; immutable baseline provides serialization, adapter and validation boundaries only. Original arithmetic is independently authored.",
        excluded="No candidate implementation or binary, product execution, live application, private reference container, screenshot, assembly or proprietary extract.",
    ))


if __name__ == "__main__":
    main()
