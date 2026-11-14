"""Create original fixtures and freeze independent expectations; run before candidates."""
from copy import deepcopy
from decimal import getcontext
from pathlib import Path
import hashlib
import json
import math

from oracle import Geometry, group_expectation, independent_checks

ROOT = Path(__file__).resolve().parent
getcontext().prec = 60


def write(name, value):
    path = ROOT / name
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, allow_nan=False) + "\n")


def project():
    return {"revision": 73, "points": [], "curves": [], "pieces": [],
            "materials": [{"id": 1, "name": "Original oracle cloth; uncalibrated",
                           "areal_density": 0.18, "warp_stiffness": 400.0,
                           "weft_stiffness": 400.0, "shear_stiffness": 100.0,
                           "bending_stiffness": 0.003, "damping": 0.2,
                           "friction": 0.4, "collision_thickness": 0.003,
                           "color": [0.25, 0.5, 0.75]}],
            "seams": [], "sewing_groups": [], "mannequin": {"capsules": []},
            "resolution": 0.11, "next_id": 1001}


def piece(p, index, start, end, controls=None):
    first = 2 + 9 * index
    point_ids = list(range(first + 1, first + 5))
    curve_ids = list(range(first + 5, first + 9))
    upper = max(start[1], end[1], *(q[1] for q in (controls or []))) + 0.6
    positions = [start, end, [end[0], upper], [start[0], upper]]
    p["points"].extend({"id": i, "position": position} for i, position in zip(point_ids, positions))
    p["curves"].extend({"id": i, "start": point_ids[k], "end": point_ids[(k + 1) % 4],
                        "controls": controls if k == 0 else None} for k, i in enumerate(curve_ids))
    p["pieces"].append({"id": first, "name": f"Original disconnected panel {index + 1}",
                         "boundary": curve_ids, "placement": {"translation": [0.0, 0.0, 0.0],
                                                                 "rotation": [0.0, 0.0, 0.0]},
                         "material": 1, "grain_angle": 0.0})
    return curve_ids[0]


def interval(curve, t0, t1):
    return {"curve": curve, "t0": t0, "t1": t1}


def document(p, version=3):
    return {"format": "weftforge", "version": version, "project": p}


def line_project():
    p = project()
    curves = [piece(p, i, start, end) for i, (start, end) in enumerate([
        ([0.0, 0.0], [0.4, 0.0]), ([1.0, 1.0], [1.5, 1.0]),
        ([2.0, 0.2], [2.4, 0.2]), ([3.0, 1.3], [3.6, 1.3]),
        ([4.2, -0.1], [4.6, -0.1])])]
    p["sewing_groups"] = [{"id": 1000, "name": "Original disconnected two to three",
                           "a": [interval(curves[0], 0.125, 0.375), interval(curves[1], 0.9, 0.3)],
                           "b": [interval(curves[2], 0.25, 0.75), interval(curves[3], 0.75, 0.25),
                                 interval(curves[4], 0.125, 0.375)],
                           "correspondence": [[0.0, 0.0], [1.0, 1.0]]}]
    return document(p)


def collinear_project():
    p = project()
    a = piece(p, 0, [-0.0, 0.0], [0.7, 0.0], [[0.01, 0.0], [0.05, 0.0]])
    b = piece(p, 1, [1.0, 0.1], [1.6, 0.1])
    p["sewing_groups"] = [{"id": 1000, "name": "Nonuniform collinear authored speed",
                           "a": [interval(a, 0.9, 0.2)], "b": [interval(b, 0.1, 0.95)],
                           "correspondence": [[0.0, 0.0], [0.23, 0.41], [0.71, 0.82], [1.0, 1.0]]}]
    return document(p)


def asymmetric_project():
    p = project()
    a = piece(p, 0, [-0.4, 0.2], [1.05, 0.39], [[0.05, 0.71], [0.68, -0.17]])
    a2 = piece(p, 1, [1.6, 1.0], [2.08, 1.0])
    b = piece(p, 2, [2.8, 0.1], [3.7, 0.33], [[3.0, -0.45], [3.32, 0.88]])
    b2 = piece(p, 3, [4.0, 1.2], [4.62, 1.2])
    b3 = piece(p, 4, [5.0, -0.1], [5.8, -0.1], [[5.02, -0.1], [5.53, -0.1]])
    p["sewing_groups"] = [{"id": 1000, "name": "Asymmetric cubic two to three",
                           "a": [interval(a, 0.13, 0.87), interval(a2, 0.8, 0.25)],
                           "b": [interval(b, 0.94, 0.17), interval(b2, 0.2, 0.7), interval(b3, 0.83, 0.29)],
                           "correspondence": [[0.0, 0.0], [0.18, 0.29], [0.57, 0.63], [1.0, 1.0]]}]
    return document(p)


def legacy_project():
    d = collinear_project()
    d["version"] = 2
    p = d["project"]
    p["revision"] = 1099511627779
    group = p.pop("sewing_groups")[0]
    group["name"] = "Legacy normalized parameter map; do not reinterpret"
    group["a"] = group["a"][0]
    group["b"] = group["b"][0]
    group["a"]["t0"] = math.nextafter(0.9, 0.0)
    group["b"]["t0"] = math.nextafter(0.1, 1.0)
    group["correspondence"] = [[0.0, 0.0], [math.nextafter(0.2, 0.0), math.nextafter(0.1, 1.0)],
                               [math.nextafter(0.55, 1.0), math.nextafter(0.7, 0.0)], [1.0, 1.0]]
    p["seams"] = [group]
    p["resolution"] = math.nextafter(0.07, 1.0)
    p["points"][4]["position"][0] = math.nextafter(1.0, 2.0)
    p["curves"][0]["controls"][0][0] = math.nextafter(0.01, 1.0)
    p["pieces"][0]["grain_angle"] = -0.0
    p["pieces"][1]["grain_angle"] = math.nextafter(0.1234567890123456, 1.0)
    p["materials"][0]["warp_stiffness"] = 456.1234567890123
    p["materials"][0]["weft_stiffness"] = 321.9876543210123
    return d


def native_migration(d):
    result = deepcopy(d)
    result["version"] = 3
    p = result["project"]
    p["sewing_groups"] = []
    if d["version"] == 1:
        for material in p["materials"]:
            material["warp_stiffness"] = material["weft_stiffness"] = material.pop("stretch_stiffness")
        for panel in p["pieces"]:
            panel["grain_angle"] = 0.0
    return result


def main():
    cases = {"line-identity-coarse": line_project(), "line-nonlinear-coarse": line_project(),
             "collinear-coarse": collinear_project(), "asymmetric-coarse": asymmetric_project(),
             "legacy-v2-f64": legacy_project()}
    cases["line-nonlinear-coarse"]["project"]["sewing_groups"][0]["correspondence"] = [
        [0.0, 0.0], [0.2, 0.1], [0.55, 0.7], [1.0, 1.0]]
    for base in ("line-identity", "line-nonlinear", "collinear", "asymmetric"):
        fine = deepcopy(cases[base + "-coarse"])
        fine["project"]["resolution"] = 0.017
        fine["project"]["revision"] += 1
        cases[base + "-fine"] = fine
    legacy_v1 = deepcopy(cases["legacy-v2-f64"])
    legacy_v1["version"] = 1
    for material in legacy_v1["project"]["materials"]:
        material["stretch_stiffness"] = material.pop("warp_stiffness")
        material.pop("weft_stiffness")
    for panel in legacy_v1["project"]["pieces"]:
        panel.pop("grain_angle")
    cases["legacy-v1-f64"] = legacy_v1
    expectation = {"schema": "independent-chain-sewing-expectations-v1", "budget_m": "1e-8", "cases": {}}
    for name, d in cases.items():
        write(f"fixtures/{name}.weftforge", d)
        expected_source = d if d["version"] == 3 else native_migration(d)
        geometry = Geometry(expected_source)
        records = [group_expectation(geometry, g) for g in expected_source["project"]["sewing_groups"]]
        expectation["cases"][name] = {"fixture": f"fixtures/{name}.weftforge", "source": expected_source,
                                        "sewing_groups": records,
                                        "quadrature_cross_checks": independent_checks(expected_source)}
    write("expectations.json", expectation)
    write("fixtures/empty-groups-v3.weftforge", {**deepcopy(cases["line-identity-coarse"]),
          "project": {**deepcopy(cases["line-identity-coarse"]["project"]), "sewing_groups": [],
                      "revision": 72, "next_id": 1000}})
    base = cases["line-identity-coarse"]
    negative = {}
    def invalid(name, mutate):
        d = deepcopy(base)
        mutate(d)
        negative[name] = d
    invalid("missing-v3-groups", lambda d: d["project"].pop("sewing_groups"))
    invalid("unknown-group-field", lambda d: d["project"]["sewing_groups"][0].update({"unknown": 1}))
    invalid("future-version", lambda d: d.update({"version": 4}))
    invalid("empty-side", lambda d: d["project"]["sewing_groups"][0].update({"a": []}))
    invalid("unknown-curve", lambda d: d["project"]["sewing_groups"][0]["a"][0].update({"curve": 999999}))
    invalid("zero-span", lambda d: d["project"]["sewing_groups"][0]["a"][0].update({"t1": 0.125}))
    invalid("overlap-within-side", lambda d: d["project"]["sewing_groups"][0]["a"].append(interval(7, 0.2, 0.6)))
    invalid("overlap-across-sides", lambda d: d["project"]["sewing_groups"][0]["b"].append(interval(7, 0.2, 0.6)))
    invalid("nonmonotone-map", lambda d: d["project"]["sewing_groups"][0].update({"correspondence": [[0.0, 0.0], [0.4, 0.8], [0.7, 0.6], [1.0, 1.0]]}))
    invalid("duplicate-group-id", lambda d: d["project"]["sewing_groups"].append(deepcopy(d["project"]["sewing_groups"][0])))
    invalid("oversized-side", lambda d: d["project"]["sewing_groups"][0].update({"a": [interval(7, k / 1025, (k + 1) / 1025) for k in range(1025)]}))
    def too_many_references(d):
        group = deepcopy(d["project"]["sewing_groups"][0])
        group["a"] = [interval(7, k / 1024, (k + 1) / 1024) for k in range(1024)]
        group["b"] = [interval(25, 0.25, 0.75)]
        d["project"]["sewing_groups"] = [{**deepcopy(group), "id": 1000 + k} for k in range(20)]
        d["project"]["next_id"] = 1020
    invalid("total-interval-bound", too_many_references)
    for name, d in negative.items():
        write(f"fixtures/invalid-{name}.weftforge", d)
    duplicate = json.dumps(base, indent=2).replace('"version": 3,', '"version": 3, "version": 3,', 1)
    (ROOT / "fixtures/invalid-duplicate-json-field.weftforge").write_text(duplicate + "\n")
    write("negative-cases.json", {"must_reject": list(negative) + ["duplicate-json-field"],
          "preserve": ["construction", "revision", "history/Redo", "old destination bytes"],
          "expensive_work": {"status": "prospective", "case": "cancel before install, stale revision, explicit exhausted-work injection",
                              "required": "use bounded job/public test configuration; useful error; no approximate fallback/partial install"}})
    files = [p for p in ROOT.rglob("*") if p.is_file() and p.suffix != ".pyc" and "results" not in p.parts and p.name != "freeze.json"]
    print(json.dumps({"prepared": len(cases), "negative": len(negative) + 1, "files": len(files)}))


if __name__ == "__main__":
    main()
