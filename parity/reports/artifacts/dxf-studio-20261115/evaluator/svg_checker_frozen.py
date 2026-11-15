#!/usr/bin/env python3
"""Independent, fixture-specific static check; never imports WeftForge code."""
import argparse
import copy
import hashlib
import json
import math
from pathlib import Path
import re
import sys
import tempfile
import xml.etree.ElementTree as ET

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
ORIGINALS = ROOT / ".local/native-svg-acceptance-20261115"
SPEC = ROOT / ".local/svg-studio-integration-20261115/spec/features/FILE-005-svg-pattern-import.md"
HASHES = {
    "baseline.weftforge": "1ec589e21fe616ef0145bdab7f830283c5c5502a982ad718d6da762566e5278f",
    "original-editable-patterns.svg": "0c85ec3b00665cc668c3dbcc32b0d35e7ba73d0cb26d19a25763ae27426f3676",
    "unsupported-transform.svg": "a95bb5bbf2b14284029abe653fbf220e6d6b6d9563a731dd586bb517b8af78ac",
}
POINT_TOLERANCE = 1e-12
SAMPLE_TOLERANCE = 1e-8
NUMBER = r"[-+]?(?:\d+\.\d*|\.\d+|\d+)(?:[eE][-+]?\d+)?"
TOKEN = re.compile(r"[MLCZ]|" + NUMBER)
NS = "{http://www.w3.org/2000/svg}"


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, f"duplicate JSON key: {key}")
        result[key] = value
    return result


def load_json(path):
    return json.loads(Path(path).read_text(encoding="utf-8"),
                      object_pairs_hook=unique_object,
                      parse_constant=lambda token: require(False, f"invalid JSON constant: {token}"))


def exact(actual, expected, label):
    require(type(actual) is type(expected), f"{label}: changed JSON type")
    if isinstance(expected, dict):
        require(actual.keys() == expected.keys(), f"{label}: changed fields")
        for key in expected:
            exact(actual[key], expected[key], f"{label}.{key}")
    elif isinstance(expected, list):
        require(len(actual) == len(expected), f"{label}: changed item count")
        for i, (a, b) in enumerate(zip(actual, expected)):
            exact(a, b, f"{label}[{i}]")
    else:
        require(actual == expected, f"{label}: {actual!r} != {expected!r}")


def pair(value, label):
    require(isinstance(value, (list, tuple)) and len(value) == 2, f"{label}: invalid coordinate pair")
    require(all(type(n) in (int, float) and math.isfinite(n) for n in value), f"{label}: nonfinite coordinate")
    return tuple(value)


def near(actual, expected, tolerance, label):
    a, b = pair(actual, label), pair(expected, label)
    require(math.hypot(a[0] - b[0], a[1] - b[1]) <= tolerance, f"{label}: {a} != {b}")


def tokens(data):
    result, position = [], 0
    while position < len(data):
        if data[position].isspace() or data[position] == ",":
            position += 1
            continue
        match = TOKEN.match(data, position)
        require(match is not None, f"unsupported/invalid path token at {position}")
        result.append(match.group())
        position = match.end()
    return result


def parse_path(data):
    # shortcut: only the predeclared absolute M/L/C/Z profile, widen for a new profile before observing outputs.
    stream, index, command = tokens(data), 0, None
    start = current = None
    spans, closed = [], False
    while index < len(stream):
        if stream[index] in ("M", "L", "C", "Z"):
            command = stream[index]
            index += 1
        require(command is not None and not closed, "missing command or content after closepath")
        if command == "Z":
            require(current is not None, "closepath without moveto")
            if current != start:
                spans.append((current, None, start))
            closed = True
            continue
        count = 6 if command == "C" else 2
        group = stream[index:index + count]
        require(len(group) == count and all(re.fullmatch(NUMBER, n) for n in group), "incomplete coordinate group")
        numbers = [float(n) for n in group]
        require(all(math.isfinite(n) for n in numbers), "nonfinite path coordinate")
        index += count
        end = tuple(numbers[-2:])
        if command == "M":
            require(start is None, "multiple subpaths")
            start = current = end
            command = "L"
            continue
        require(current is not None, "span before moveto")
        controls = (tuple(numbers[:2]), tuple(numbers[2:4])) if command == "C" else None
        spans.append((current, controls, end))
        current = end
    require(closed and len(spans) >= 3, "missing explicit closepath or too few spans")
    return spans


def read_svg(data):
    text = data.decode("utf-8") if isinstance(data, bytes) else data
    require("<!DOCTYPE" not in text and "<!ENTITY" not in text, "DTD/entity not permitted")
    root = ET.fromstring(text)
    require(root.tag == NS + "svg", "wrong SVG namespace/root")
    require(not any("transform" in node.attrib for node in root.iter()), "unexpected SVG transform")
    lengths = []
    for key in ("width", "height"):
        match = re.fullmatch(r"(" + NUMBER + r")mm", root.get(key, ""))
        require(match is not None, f"{key} must have explicit mm units")
        lengths.append(float(match.group(1)))
    box = tokens(root.get("viewBox", ""))
    require(len(box) == 4 and all(re.fullmatch(NUMBER, n) for n in box), "invalid viewBox")
    box = list(map(float, box))
    require(all(math.isfinite(n) for n in lengths + box) and min(lengths + box[2:]) > 0, "invalid physical scale")
    sx, sy = lengths[0] / box[2] / 1000, lengths[1] / box[3] / 1000
    require(math.isclose(sx, sy, rel_tol=1e-12, abs_tol=0), "nonuniform SVG scale")

    def physical(p):
        return (p[0] * sx, -p[1] * sx)

    paths = []
    for child in root:
        if child.tag in (NS + "title", NS + "desc"):
            continue
        require(child.tag == NS + "path", f"unsupported export element: {child.tag}")
        name = child.get("data-name", child.get("id", f"SVG pattern {len(paths) + 1}"))
        spans = [(physical(a), None if c is None else tuple(map(physical, c)), physical(b))
                 for a, c, b in parse_path(child.get("d", ""))]
        paths.append((name, spans))
    return paths, {"metres_per_user_unit": sx, "view_box": box, "origin_policy": "user-space origin retained; no viewBox-min subtraction"}


def originals(directory=ORIGINALS):
    for name, expected_hash in HASHES.items():
        require(digest(directory / name) == expected_hash, f"original input bytes changed: {name}")
    return load_json(directory / "baseline.weftforge")


def expected_geometry(baseline):
    source, scale = read_svg((ORIGINALS / "original-editable-patterns.svg").read_bytes())
    require(scale["metres_per_user_unit"] == .002, "original source scale changed")
    analytic = [
        ("Original & curved front", [((-.16, .32), None, (.04, .32)),
         ((.04, .32), ((.08, .32), (.08, .20)), (.04, .20)),
         ((.04, .20), None, (-.16, .20)), ((-.16, .20), None, (-.16, .32))]),
        ("back-panel", [((.12, .30), None, (.32, .30)), ((.32, .30), None, (.32, .10)),
         ((.32, .10), None, (.12, .10)), ((.12, .10), None, (.12, .30))]),
    ]
    compare_geometry(source, analytic, "original fixture analytic cross-check")
    old = copy.deepcopy(baseline)
    project = old["project"]
    left, right = min(p["position"][0] for p in project["points"]), max(p["position"][0] for p in project["points"])
    require(math.isclose(right - left, .1, abs_tol=1e-15), "baseline is not declared .1 m rectangle")
    for point in project["points"]:
        if point["position"][0] == right:
            point["position"][0] = left + .125
    original = native_geometry(project)
    edited = copy.deepcopy(analytic)
    edited[0][1][1] = ((.04, .32), ((.09, .32), (.08, .20)), (.04, .20))
    return old, original + edited, scale


def id_map(objects, label):
    result = {}
    for obj in objects:
        require(type(obj.get("id")) is int and obj["id"] > 0 and obj["id"] not in result, f"{label}: invalid/duplicate ID")
        result[obj["id"]] = obj
    return result


def native_geometry(project):
    points, curves = id_map(project["points"], "points"), id_map(project["curves"], "curves")
    result = []
    for piece in project["pieces"]:
        boundary = piece["boundary"]
        require(isinstance(boundary, list) and all(type(i) is int for i in boundary), "boundary IDs must be integers")
        require(len(boundary) >= 3 and len(set(boundary)) == len(boundary), "invalid boundary IDs")
        spans = []
        for i, curve_id in enumerate(boundary):
            require(curve_id in curves, "missing boundary curve")
            curve, following = curves[curve_id], curves[boundary[(i + 1) % len(boundary)]]
            require(type(curve["start"]) is int and type(curve["end"]) is int, "endpoint IDs must be integers")
            require(curve["end"] == following["start"], "broken closed boundary ID connectivity")
            require(curve["start"] in points and curve["end"] in points, "missing curve endpoint")
            spans.append((points[curve["start"]]["position"], curve["controls"], points[curve["end"]]["position"]))
        result.append((piece["name"], spans))
    return result


def sample(span, t):
    a, controls, b = span
    if controls is None:
        return tuple((1 - t) * a[i] + t * b[i] for i in range(2))
    c, d = controls
    return tuple((1 - t) ** 3 * a[i] + 3 * (1 - t) ** 2 * t * c[i] +
                 3 * (1 - t) * t ** 2 * d[i] + t ** 3 * b[i] for i in range(2))


def compare_geometry(actual, expected, label):
    require(len(actual) == len(expected), f"{label}: wrong path/piece count")
    for i, ((name, spans), (wanted_name, wanted_spans)) in enumerate(zip(actual, expected)):
        require(name == wanted_name, f"{label}: piece {i} name/order changed")
        require(len(spans) == len(wanted_spans), f"{label}: piece {i} span count changed")
        for j, (span, wanted) in enumerate(zip(spans, wanted_spans)):
            near(span[0], wanted[0], POINT_TOLERANCE, f"{label} piece {i} span {j} start")
            near(span[2], wanted[2], POINT_TOLERANCE, f"{label} piece {i} span {j} end")
            require((span[1] is None) == (wanted[1] is None), f"{label}: cubic/line type changed")
            if span[1] is not None:
                require(len(span[1]) == 2, "invalid cubic controls")
                for k in range(2):
                    near(span[1][k], wanted[1][k], POINT_TOLERANCE, f"{label} piece {i} span {j} control {k}")
            for k in range(17):
                near(sample(span, k / 16), sample(wanted, k / 16), SAMPLE_TOLERANCE, f"{label} piece {i} span {j} sample {k}/16")


def check_native(actual, baseline, expected_old, geometry):
    require(actual.keys() == baseline.keys(), "native envelope fields changed")
    for key in baseline.keys() - {"project"}:
        exact(actual[key], baseline[key], key)
    a, old, original = actual["project"], expected_old["project"], baseline["project"]
    require(a.keys() == original.keys(), "native project fields changed")
    for key in original.keys() - {"revision", "next_id", "points", "curves", "pieces"}:
        exact(a[key], original[key], f"project.{key}")
    all_ids = []
    for key, additions in (("points", 8), ("curves", 8), ("pieces", 2)):
        require(len(a[key]) == len(old[key]) + additions, f"{key}: unexpected object count")
        exact(a[key][:len(old[key])], old[key], f"preserved {key}")
        id_map(a[key], key)
        for obj in a[key][len(old[key]):]:
            require(obj.keys() == old[key][0].keys(), f"new {key}: unexpected authoring fields")
            require(obj["id"] >= original["next_id"], f"new {key}: preexisting ID reused")
        all_ids.extend(obj["id"] for obj in a[key])
    all_ids.extend(material["id"] for material in a["materials"])
    require(len(all_ids) == len(set(all_ids)), "cross-kind stable ID collision")
    require(type(a["next_id"]) is int and a["next_id"] > max(all_ids), "invalid stable-ID allocator")
    require(type(a["revision"]) is int and a["revision"] > original["revision"], "invalid/unchanged native revision")
    for piece in a["pieces"][len(original["pieces"]):]:
        exact(piece["material"], original["materials"][0]["id"], "import default material")
        exact(piece["grain_angle"], 0.0, "import default grain")
        exact(piece["placement"], {"translation": [0.0] * 3, "rotation": [0.0] * 3}, "import default placement")
    used_curves = [c for p in a["pieces"] for c in p["boundary"]]
    require(len(used_curves) == len(set(used_curves)) and set(used_curves) == {c["id"] for c in a["curves"]}, "extra/shared/missing boundary curves")
    used_points = {p for c in a["curves"] for p in (c["start"], c["end"])}
    require(used_points == {p["id"] for p in a["points"]}, "extra/missing points")
    compare_geometry(native_geometry(a), geometry, "native authoring geometry")


def check_reopen(saved, resaved, allowed_delta=0, revision_note=None):
    if allowed_delta:
        require(bool(revision_note and revision_note.strip()), "revision exception needs a documented reason")
    adjusted = copy.deepcopy(resaved)
    require(type(adjusted["project"]["revision"]) is int, "resaved revision is not integer")
    require(adjusted["project"]["revision"] - saved["project"]["revision"] == allowed_delta, "unexpected quit/reopen/resave revision difference")
    adjusted["project"]["revision"] = saved["project"]["revision"]
    exact(adjusted, saved, "quit/reopen/resave authoring document")


def synthetic_native(baseline, expected_old, geometry):
    result, next_id = copy.deepcopy(expected_old), 1001
    p = result["project"]
    for name, spans in geometry[len(p["pieces"]):]:
        point_ids, curve_ids = [], []
        for start, _, _ in spans:
            point_ids.append(next_id)
            p["points"].append({"id": next_id, "position": list(start)})
            next_id += 7
        for i, (_, controls, _) in enumerate(spans):
            curve_ids.append(next_id)
            p["curves"].append({"id": next_id, "start": point_ids[i], "end": point_ids[(i + 1) % len(point_ids)],
                                "controls": None if controls is None else [list(c) for c in controls]})
            next_id += 7
        p["pieces"].append({"id": next_id, "name": name, "boundary": curve_ids,
                             "placement": {"translation": [0.0] * 3, "rotation": [0.0] * 3},
                             "material": p["materials"][0]["id"], "grain_angle": 0.0})
        next_id += 7
    p["revision"], p["next_id"] = baseline["project"]["revision"] + 2, next_id
    return result


def synthetic_svg(geometry):
    root = ET.Element(NS + "svg", width="1000mm", height="1000mm", viewBox="-1000 -1000 1000 1000")
    def xy(p):
        return f"{p[0] * 1000:.17g} {-p[1] * 1000:.17g}"
    for name, spans in geometry:
        data = "M" + xy(spans[0][0])
        for _, controls, end in spans:
            data += ("L" if controls is None else "C" + xy(controls[0]) + " " + xy(controls[1]) + " ") + xy(end)
        ET.SubElement(root, NS + "path", {"data-name": name, "d": data + "Z"})
    return ET.tostring(root, encoding="utf-8")


def self_checks(baseline, old, geometry):
    good = synthetic_native(baseline, old, geometry)
    check_native(good, baseline, old, geometry)
    compare_geometry(read_svg(synthetic_svg(geometry))[0], geometry, "synthetic SVG")
    check_reopen(good, copy.deepcopy(good))
    rejected = []

    def reject(label, action):
        try:
            action()
        except (ValueError, KeyError, TypeError, IndexError):
            rejected.append(label)
        else:
            raise ValueError(f"self-check failed to reject {label}")

    def mutate(action):
        wrong = copy.deepcopy(good)
        action(wrong["project"])
        check_native(wrong, baseline, old, geometry)

    reject("wrong import scale", lambda: mutate(lambda p: p["points"][4]["position"].__setitem__(0, -.08)))
    reject("viewBox origin subtraction", lambda: mutate(lambda p: p["points"][4]["position"].__setitem__(0, .04)))
    reject("missing cubic edit", lambda: mutate(lambda p: p["curves"][5]["controls"][0].__setitem__(0, .08)))
    reject("wrong cubic control", lambda: mutate(lambda p: p["curves"][5]["controls"][0].__setitem__(0, .091)))
    reject("omitted original piece", lambda: mutate(lambda p: p["pieces"].pop(0)))
    reject("hidden material change", lambda: mutate(lambda p: p["materials"][0].__setitem__("friction", .5)))
    altered_svg = synthetic_svg(geometry).replace(b'width="1000mm" height="1000mm"', b'width="2000mm" height="2000mm"')
    reject("wrong export physical scale", lambda: compare_geometry(read_svg(altered_svg)[0], geometry, "wrong scale"))
    wrong_reopen = copy.deepcopy(good)
    wrong_reopen["project"]["points"][0]["position"][0] += 1e-6
    reject("reopen authoring change", lambda: check_reopen(good, wrong_reopen))
    for changed in ("baseline.weftforge", "original-editable-patterns.svg", "unsupported-transform.svg"):
        with tempfile.TemporaryDirectory(prefix="weftforge-svg-self-check-") as temporary:
            directory = Path(temporary)
            for name in HASHES:
                (directory / name).write_bytes((ORIGINALS / name).read_bytes() + (b"\n" if name == changed else b""))
            reject(f"original byte edit: {changed}", lambda: originals(directory))
    reject("relative SVG command", lambda: parse_path("M0 0 l1 0 L1 1 Z"))
    reject("unclosed SVG path", lambda: parse_path("M0 0 L1 0 L1 1"))
    return {"status": "SELF_CHECKS_PASSED", "native_workflow_acceptance": "NOT_RUN", "synthetic_positive_case": True,
            "rejected_cases": rejected, "note": "Only checker self-tests; no actual native output was read or graded."}


def frozen():
    manifest = load_json(HERE / "FROZEN.json")
    for name, expected_hash in manifest["evaluator_sha256"].items():
        require(digest(HERE / name) == expected_hash, f"frozen evaluator changed: {name}")
    require(digest(SPEC) == manifest["spec_sha256"], "prospective specification bytes changed")
    return manifest


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-check", action="store_true")
    parser.add_argument("--saved", type=Path)
    parser.add_argument("--resaved", type=Path)
    parser.add_argument("--exported-svg", type=Path)
    parser.add_argument("--allowed-resave-revision-delta", type=int, default=0)
    parser.add_argument("--revision-note")
    args = parser.parse_args()
    baseline = originals()
    old, geometry, source_scale = expected_geometry(baseline)
    if args.self_check:
        require(not any((args.saved, args.resaved, args.exported_svg)), "self-check cannot grade actual outputs")
        return self_checks(baseline, old, geometry)
    require(all((args.saved, args.resaved, args.exported_svg)), "actual acceptance requires --saved, --resaved and --exported-svg; missing outputs never pass")
    manifest = frozen()
    paths = [args.saved, args.resaved, args.exported_svg]
    require(len({p.resolve() for p in paths}) == 3, "saved/resaved/export must be separate artifacts")
    require(not {p.resolve() for p in paths} & {(ORIGINALS / name).resolve() for name in HASHES}, "actual artifacts alias original sources")
    saved, resaved = load_json(args.saved), load_json(args.resaved)
    check_native(saved, baseline, old, geometry)
    check_native(resaved, baseline, old, geometry)
    check_reopen(saved, resaved, args.allowed_resave_revision_delta, args.revision_note)
    exported, export_scale = read_svg(args.exported_svg.read_bytes())
    compare_geometry(exported, geometry, "independent exported SVG")
    originals()
    return {"status": "STATIC_ARTIFACT_CHECKS_PASSED", "native_action_execution": "Requires separate native action trace review; not established by this static checker",
            "frozen_utc": manifest["frozen_utc"], "evaluator_sha256": manifest["evaluator_sha256"],
            "original_input_sha256": HASHES, "actual_artifact_sha256": {str(p): digest(p) for p in paths},
            "piece_names_in_order": [name for name, _ in geometry], "piece_count": 3, "spans_per_piece": [len(s) for _, s in geometry],
            "source_scale": source_scale, "export_scale": export_scale, "endpoint_control_tolerance_m": POINT_TOLERANCE,
            "sample_tolerance_m": SAMPLE_TOLERANCE, "samples_per_span": 17, "saved_revision": saved["project"]["revision"],
            "resaved_revision": resaved["project"]["revision"], "allowed_resave_revision_delta": args.allowed_resave_revision_delta,
            "revision_note": args.revision_note, "preserved_fields": "All original authoring objects except declared anchored rectangle width; all saved/resaved fields exact except explicitly documented revision delta"}


if __name__ == "__main__":
    try:
        report = main()
    except (OSError, ValueError, KeyError, TypeError, IndexError, ET.ParseError) as error:
        print(json.dumps({"status": "FAILED", "error": str(error), "native_workflow_acceptance": "NOT_ESTABLISHED"}, indent=2))
        sys.exit(1)
    print(json.dumps(report, indent=2))
