#!/usr/bin/env python3
"""Read saved v3 documents only; native interaction evidence is reviewed separately."""
import copy
import hashlib
import importlib.util
import json
import math
from pathlib import Path
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
LAB = ROOT.parent / "transform-independent-evaluation-20261115"
PINS = {
    "evaluate.py": "5dbb1e7299b38df921db4d2004017e5e62ddf9eb3275a5afbdc28d49d0c58e8f",
    "fixture.weftforge": "a7c7a096f1b1db531a3fb4a2770352f65643856450b52b1856a7db61ef20bc59",
    "plan.json": "968ba802e404e731d5159638122993c22ca0f0f7d4ac4b254d5aab286a05aea9",
}


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


for name, digest in PINS.items():
    if sha(LAB / name) != digest:
        raise AssertionError("frozen independent artifact changed: " + name)
spec = importlib.util.spec_from_file_location("frozen_transform_acceptance", LAB / "evaluate.py")
F = importlib.util.module_from_spec(spec)
spec.loader.exec_module(F)
F.frozen_files()

QUARTER = {"piece": 2, "translation": [0.5, -0.25], "angle": math.pi / 2,
           "pivot": [0.25, 0.5]}
TRANSLATE = {"piece": 2, "translation": [0.10, 0.05], "angle": 0.0,
             "pivot": [0.625, 0.125]}
ROTATION_PIVOT = [0.725, 0.175]
ANGLE_WINDOW = [math.radians(43), math.radians(47)]


def fresh_revision(document, revision):
    result = copy.deepcopy(document)
    result["project"]["revision"] = revision
    return result


def grade_transform(before, after, command, case):
    envelope_before, envelope_after = copy.deepcopy(before), copy.deepcopy(after)
    envelope_before["project"] = envelope_after["project"] = None
    F.exact(envelope_before, envelope_after, "v3_envelope")
    return F.grade(before, after, command, case)


def grade_quarter(document, revision):
    F.require(document["project"]["revision"] == revision, "expected quarter revision")
    return grade_transform(fresh_revision(F.FIXTURE, revision - 1), document,
                           QUARTER, {"oracle": "quarter"})


def grade_a(quarter, undo, redo, reopened):
    result = {"quarter": grade_quarter(quarter, 38)}
    F.exact(fresh_revision(F.FIXTURE, 39), undo, "undo_restores_every_authored_bit")
    result["redo"] = grade_quarter(redo, 40)
    F.exact(fresh_revision(quarter, 40), redo, "redo_restores_every_authored_bit")
    result["reopened"] = grade_quarter(reopened, 40)
    F.exact(redo, reopened, "same_process_reopen_exact_fields")
    return result


def grade_b(redo, translated, rotated):
    grade_quarter(redo, 40)
    result = {"translation": grade_transform(redo, translated, TRANSLATE,
                                              {"source_is_transformed": True})}
    # shortcut: two endpoints fit the observed gesture angle, not a pixel mapping; use event tests for strict screen calibration.
    before, after = (document["project"] for document in (translated, rotated))
    before_points = [F.by_id(before, "points", i)["position"] for i in (3, 4)]
    after_points = [F.by_id(after, "points", i)["position"] for i in (3, 4)]
    u = [before_points[1][i] - before_points[0][i] for i in range(2)]
    v = [after_points[1][i] - after_points[0][i] for i in range(2)]
    F.near([math.hypot(*u)], [math.hypot(*v)], 1e-10, "fit endpoints retain span")
    angle = math.atan2(u[0] * v[1] - u[1] * v[0], u[0] * v[0] + u[1] * v[1])
    F.require(ANGLE_WINDOW[0] <= angle <= ANGLE_WINDOW[1], "rotation outside fixed 43..47 degree window")
    command = {"piece": 2, "translation": [0.0, 0.0], "angle": angle,
               "pivot": ROTATION_PIVOT}
    result["rotation"] = grade_transform(translated, rotated, command,
                                          {"source_is_transformed": True})
    result["fitted_angle_rad"] = angle
    result["fitted_angle_deg"] = math.degrees(angle)
    result["coverage"] = "coarse real-device gesture only; no strict screen-to-pattern mapping claim"
    return result


def synthetic(before, command, oracle=None):
    """Evaluator self-test data only, derived from independent matrix multiplication."""
    a = F.rotation(F.by_id(before["project"], "pieces", 2)["placement"]["rotation"])
    matrix = F.multiply(a, F.rotation([0.0, 0.0, -command["angle"]]))
    euler = (copy.deepcopy(F.by_id(before["project"], "pieces", 2)["placement"]["rotation"])
             if command["angle"] == 0.0 else
             [math.atan2(matrix[2][1], matrix[2][2]),
              math.atan2(-matrix[2][0], math.hypot(matrix[0][0], matrix[1][0])),
              math.atan2(matrix[1][0], matrix[0][0])])
    return F.synthetic_layout(before, command, euler, oracle)


def selfcheck():
    quarter = synthetic(F.FIXTURE, QUARTER, "quarter")
    undo, redo = fresh_revision(F.FIXTURE, 39), fresh_revision(quarter, 40)
    reopened = copy.deepcopy(redo)
    grade_a(quarter, undo, redo, reopened)
    translated = synthetic(redo, TRANSLATE)
    rotate = {"piece": 2, "translation": [0.0, 0.0], "angle": math.pi / 4,
              "pivot": ROTATION_PIVOT}
    rotated = synthetic(translated, rotate)
    grade_b(redo, translated, rotated)
    checks = ["accepted synthetic numeric/history/reopen and 45-degree pointer documents"]

    def reject(name, invoke):
        try:
            invoke()
        except (AssertionError, ValueError, KeyError, TypeError) as problem:
            checks.append({"rejected": name, "reason": str(problem)})
        else:
            raise AssertionError("bad-output self-check accepted " + name)

    a_mutations = [
        ("quarter_control_not_rotated", 0, lambda d: F.by_id(d["project"], "curves", 7)["controls"][0].__setitem__(0, 0.25)),
        ("quarter_missing_placement_compensation", 0, lambda d: F.by_id(d["project"], "pieces", 2).__setitem__("placement", copy.deepcopy(F.by_id(F.FIXTURE["project"], "pieces", 2)["placement"]))),
        ("quarter_unchanged_grain", 0, lambda d: F.by_id(d["project"], "pieces", 2).__setitem__("grain_angle", 0.37)),
        ("wrong_transaction_revision", 0, lambda d: d["project"].__setitem__("revision", 39)),
        ("undo_lost_signed_zero", 1, lambda d: F.by_id(d["project"], "points", 3)["position"].__setitem__(1, 0.0)),
        ("redo_changed_map", 2, lambda d: d["project"]["seams"][0]["correspondence"][1].__setitem__(1, 0.25)),
        ("redo_changed_allocator", 2, lambda d: d["project"].__setitem__("next_id", 23)),
        ("redo_changed_material", 2, lambda d: d["project"]["materials"][0].__setitem__("damping", 0.3)),
        ("reopen_lost_one_bit", 3, lambda d: F.by_id(d["project"], "points", 4)["position"].__setitem__(0, math.nextafter(0.5, 1.0))),
    ]
    for name, index, mutate in a_mutations:
        documents = copy.deepcopy([quarter, undo, redo, reopened])
        mutate(documents[index])
        reject(name, lambda: grade_a(*documents))
    wrong_translation = synthetic(redo, TRANSLATE | {"translation": [0.11, 0.05]})
    reject("wrong_snapped_displacement", lambda: grade_b(redo, wrong_translation, rotated))
    for name, command in [
        ("rotation_50_degrees", rotate | {"angle": math.radians(50)}),
        ("wrong_rotation_pivot", rotate | {"pivot": [0.75, 0.175]}),
        ("extra_rotation_translation", rotate | {"translation": [0.01, 0.0]}),
    ]:
        bad = synthetic(translated, command)
        reject(name, lambda: grade_b(redo, translated, bad))
    for name, modify in [
        ("scale", lambda q: [ROTATION_PIVOT[i] + 1.01 * (q[i] - ROTATION_PIVOT[i]) for i in range(2)]),
        ("shear", lambda q: [q[0] + 0.01 * q[1], q[1]]),
        ("reflection", lambda q: [2 * ROTATION_PIVOT[0] - q[0], q[1]]),
    ]:
        bad = copy.deepcopy(rotated)
        for point in bad["project"]["points"]:
            if point["id"] in (3, 4, 5, 6):
                point["position"] = modify(point["position"])
        curve = F.by_id(bad["project"], "curves", 7)
        curve["controls"] = [modify(q) for q in curve["controls"]]
        reject(name, lambda: grade_b(redo, translated, bad))
    bad = fresh_revision(rotated, 43)
    reject("extra_pointer_revision", lambda: grade_b(redo, translated, bad))
    report = {"status": "passed", "candidate_executed": False, "native_executed": False,
              "checks": checks, "rejected_bad_outputs": len(checks) - 1}
    (ROOT / "SELF-CHECK.json").write_text(json.dumps(report, indent=2, allow_nan=False) + "\n")
    return report


def read_document(path):
    def invalid_constant(value):
        raise ValueError("nonfinite JSON number: " + value)
    def unique_object(pairs):
        result = {}
        for key, value in pairs:
            F.require(key not in result, "duplicate JSON field: " + key)
            result[key] = value
        return result
    return json.loads(path.read_text(), parse_constant=invalid_constant,
                      object_pairs_hook=unique_object)


def main(arguments):
    if arguments == ["selfcheck"]:
        return selfcheck()
    usage = "usage: grade_saved.py selfcheck | grade-a A_READONLY_INPUT QUARTER UNDO REDO REOPENED | grade-b A_REDO B_READONLY_INPUT TRANSLATED ROTATED"
    if not arguments:
        raise SystemExit(usage)
    freeze = json.loads((ROOT / "FREEZE.json").read_text())
    for name, digest in freeze["sha256"].items():
        F.require(sha(ROOT / name) == digest, "prospective packet changed: " + name)
    paths = [Path(name).resolve() for name in arguments[1:]]
    if arguments[0] == "grade-a" and len(paths) == 5:
        readonly_input, *outputs = paths
        F.require(sha(readonly_input) == PINS["fixture.weftforge"], "A input must retain exact original bytes")
        F.require(readonly_input.stat().st_mode & 0o222 == 0, "A input must remain read-only")
        result = grade_a(*(read_document(path) for path in outputs))
    elif arguments[0] == "grade-b" and len(paths) == 4:
        redo, readonly_input, translated, rotated = paths
        F.require(redo.read_bytes() == readonly_input.read_bytes(), "B input must be byte-identical A redo")
        F.require(readonly_input.stat().st_mode & 0o222 == 0, "B input must remain read-only")
        result = grade_b(*(read_document(path) for path in (redo, translated, rotated)))
    else:
        raise SystemExit(usage)
    return {"status": "passed_saved_output_only", "ui_verified": False, "result": result,
            "sha256": {str(path): sha(path) for path in paths}}


if __name__ == "__main__":
    print(json.dumps(main(sys.argv[1:]), indent=2, allow_nan=False))
