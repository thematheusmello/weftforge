#!/usr/bin/env python3
"""Frozen, independent native rectangle file check; stdlib only."""

import copy
import hashlib
import json
import math
import os
from pathlib import Path
import stat
import sys
import tempfile
from datetime import datetime, timezone


BASE = Path(__file__).absolute().parent
ROOT = BASE.parent.parent
MAX_BYTES = 262144
EPS = 1e-12


class Rejected(ValueError):
    pass


def require(condition, message):
    if not condition:
        raise Rejected(message)


def unique_pairs(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, "duplicate JSON key: " + key)
        result[key] = value
    return result


def finite_float(text):
    value = float(text)
    require(math.isfinite(value), "nonfinite/overflow f64")
    return value


def bounded_integer(text):
    value = int(text)
    require(-(2**63) <= value <= 2**64 - 1, "integer out of native range")
    return value


def reject_constant(text):
    raise Rejected("non-JSON numeric constant: " + text)


def bounded_tree(value, depth=0):
    require(depth <= 64, "JSON nesting exceeds 64")
    if isinstance(value, dict):
        for key, child in value.items():
            bounded_tree(key, depth + 1)
            bounded_tree(child, depth + 1)
    elif isinstance(value, list):
        for child in value:
            bounded_tree(child, depth + 1)
    elif isinstance(value, str):
        require(not any(0xD800 <= ord(character) <= 0xDFFF for character in value),
                "unpaired Unicode surrogate")


def decode(raw):
    require(len(raw) <= MAX_BYTES, "file exceeds size cap")
    try:
        value = json.loads(
            raw.decode("utf-8"),
            object_pairs_hook=unique_pairs,
            parse_float=finite_float,
            parse_int=bounded_integer,
            parse_constant=reject_constant,
        )
        bounded_tree(value)
        return value
    except (UnicodeError, json.JSONDecodeError, RecursionError) as error:
        raise Rejected("invalid UTF-8/native JSON: " + str(error)) from error


def read_file(relative):
    path = Path(relative)
    require(not path.is_absolute() and path.parts and
            all(part not in (".", "..") for part in path.parts), "unsafe path")
    current = ROOT
    for part in path.parts:
        current /= part
        require(not stat.S_ISLNK(current.lstat().st_mode), "symlink refused: " + relative)
    before = current.lstat()
    require(stat.S_ISREG(before.st_mode), "not a regular file: " + relative)
    require(before.st_size <= MAX_BYTES, "file exceeds size cap: " + relative)
    descriptor = os.open(current, os.O_RDONLY | os.O_NOFOLLOW)
    with os.fdopen(descriptor, "rb") as stream:
        opened = os.fstat(stream.fileno())
        require(stat.S_ISREG(opened.st_mode), "not a regular opened file")
        require((opened.st_dev, opened.st_ino) == (before.st_dev, before.st_ino),
                "file changed while opening")
        raw = stream.read(MAX_BYTES + 1)
        after = os.fstat(stream.fileno())
    final = current.lstat()
    fields = lambda value: (value.st_dev, value.st_ino, value.st_size,
                            value.st_mode, value.st_mtime_ns, value.st_ctime_ns)
    require(fields(before) == fields(opened) == fields(after) == fields(final),
            "file changed while reading: " + relative)
    require(len(raw) <= MAX_BYTES, "file exceeds size cap: " + relative)
    return raw, {"sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw),
                 "mode": format(stat.S_IMODE(final.st_mode), "04o")}


def same(actual, expected, path="document"):
    require(type(actual) is type(expected), path + ": numeric/value type changed")
    if isinstance(expected, dict):
        require(actual.keys() == expected.keys(), path + ": field set changed")
        for key in expected:
            same(actual[key], expected[key], path + "." + key)
    elif isinstance(expected, list):
        require(len(actual) == len(expected), path + ": list length changed")
        for index, value in enumerate(expected):
            same(actual[index], value, path + "[" + str(index) + "]")
    elif isinstance(expected, float):
        require(math.isfinite(actual) and actual.hex() == expected.hex(),
                path + ": f64 bits changed")
    else:
        require(actual == expected, path + ": value changed")


def keys(value, expected, path):
    require(type(value) is dict and value.keys() == set(expected), path + ": wrong fields/type")


def vector(value, count, path):
    require(type(value) is list and len(value) == count, path + ": wrong vector length/type")
    require(all(type(number) is float and math.isfinite(number) for number in value),
            path + ": expected finite f64 coordinates")


def close(actual, expected, path):
    require(math.isfinite(actual) and abs(actual - expected) <= EPS,
            path + ": outside absolute SI tolerance")


def loop_area(positions):
    # Translate before summing so an arbitrary authoring origin does not cancel area.
    origin_x, origin_y = positions[0]
    local = [(x - origin_x, y - origin_y) for x, y in positions]
    return 0.5 * math.fsum(
        a[0] * b[1] - b[0] * a[1]
        for a, b in zip(local, local[1:] + local[:1])
    )


def check_created(document, seed):
    require(type(document) is dict and type(document.get("project")) is dict,
            "expected project envelope")
    project = document["project"]
    skeleton = copy.deepcopy(document)
    skeleton["project"].update(points=[], curves=[], pieces=[], revision=0, next_id=2)
    same(skeleton, seed)
    same(project["revision"], 1, "revision")
    same(project["next_id"], 11, "next_id")
    require(type(project["points"]) is list and len(project["points"]) == 4,
            "expected four points")
    positions = []
    for index, point in enumerate(project["points"]):
        keys(point, ("id", "position"), "point")
        same(point["id"], index + 3, "point.id")
        vector(point["position"], 2, "point.position")
        positions.append(point["position"])
        for number in point["position"]:
            close(number, round(number / 0.01) * 0.01, "10 mm lattice")
    x, y = positions[0]
    for actual, expected in zip(positions,
                                ([x, y], [x + 0.4, y], [x + 0.4, y + 0.6], [x, y + 0.6])):
        for coordinate, target in zip(actual, expected):
            close(coordinate, target, "ordered axis-aligned rectangle")
    close(positions[1][0] - positions[0][0], 0.4, "width")
    close(positions[3][1] - positions[0][1], 0.6, "height")
    area = loop_area(positions)
    close(area, 0.24, "signed loop area")
    perimeter = math.fsum(math.hypot(b[0] - a[0], b[1] - a[1])
                          for a, b in zip(positions, positions[1:] + positions[:1]))
    close(perimeter, 2.0, "perimeter")
    curves = [{"id": 7 + index, "start": 3 + index, "end": 3 + (index + 1) % 4,
               "controls": None} for index in range(4)]
    same(project["curves"], curves, "curves")
    require(type(project["pieces"]) is list and len(project["pieces"]) == 1,
            "expected one piece")
    piece = project["pieces"][0]
    keys(piece, ("id", "name", "boundary", "holes", "internal_curves", "placement",
                 "material", "grain_angle"), "piece")
    placement = piece["placement"]
    keys(placement, ("translation", "rotation"), "placement")
    vector(placement["translation"], 3, "placement.translation")
    vector(placement["rotation"], 3, "placement.rotation")
    expected = {"id": 2, "name": "Pattern 1", "boundary": [7, 8, 9, 10], "holes": [],
                "internal_curves": [], "placement": placement, "material": 1, "grain_angle": 0.0}
    same(piece, expected, "piece")
    return {"width_m": positions[1][0] - x, "height_m": positions[3][1] - y,
            "signed_area_m2": area, "perimeter_m": perimeter}


def check_history(documents, seed):
    require(type(documents) is dict and documents.keys() == {"created", "undo", "redo", "reopened-edit"},
            "wrong history stages")
    metrics = {"created": check_created(documents["created"], seed)}
    expected_undo = copy.deepcopy(seed)
    expected_undo["project"].update(revision=2, next_id=11)
    same(documents["undo"], expected_undo, "undo")
    expected_redo = copy.deepcopy(documents["created"])
    expected_redo["project"]["revision"] = 3
    same(documents["redo"], expected_redo, "redo")
    edit = documents["reopened-edit"]
    require(type(edit) is dict and type(edit.get("project")) is dict, "invalid edit envelope")
    points = edit["project"].get("points")
    require(type(points) is list and len(points) == 4, "invalid edit point count")
    keys(points[0], ("id", "position"), "edit point")
    vector(points[0]["position"], 2, "edit point.position")
    new_x = points[0]["position"][0]
    old_x = documents["redo"]["project"]["points"][0]["position"][0]
    close(new_x - old_x, -0.01, "only P0.x decreases 10 mm")
    expected_edit = copy.deepcopy(documents["redo"])
    expected_edit["project"]["revision"] = 4
    expected_edit["project"]["points"][0]["position"][0] = new_x
    same(edit, expected_edit, "reopened-edit")
    positions = [point["position"] for point in points]
    area = loop_area(positions)
    close(area, 0.243, "edited signed loop area")
    close(positions[1][0] - positions[0][0], 0.41, "edited bottom width")
    close(positions[2][0] - positions[3][0], 0.4, "edited top width")
    close(positions[3][1] - positions[0][1], 0.6, "edited height")
    for position in positions:
        vector(position, 2, "edited position")
        for number in position:
            close(number, round(number / 0.01) * 0.01, "edited 10 mm lattice")
    metrics["reopened-edit"] = {"signed_area_m2": area, "bottom_width_m": 0.41,
                                 "top_width_m": 0.4, "height_m": 0.6,
                                 "P0_x_delta_m": new_x - old_x}
    return metrics


def load_frozen_inputs():
    targets = decode(read_file(str((BASE / "targets.json").relative_to(ROOT)))[0])
    seed_raw, seed_meta = read_file(targets["seed"]["path"])
    same(seed_meta, {key: targets["seed"][key] for key in ("sha256", "bytes", "mode")}, "seed metadata")
    expected_raw = read_file(str((BASE / "seed-expected.json").relative_to(ROOT)))[0]
    require(seed_raw == expected_raw, "original seed differs from frozen seed")
    return targets, decode(seed_raw)


def synthetic_history(seed):
    created = copy.deepcopy(seed)
    project = created["project"]
    project.update(revision=1, next_id=11)
    project["points"] = [{"id": index + 3, "position": position} for index, position in enumerate(
        [[-0.37, 0.22], [0.03, 0.22], [0.03, 0.82], [-0.37, 0.82]])]
    project["curves"] = [{"id": 7 + index, "start": 3 + index, "end": 3 + (index + 1) % 4,
                          "controls": None} for index in range(4)]
    project["pieces"] = [{"id": 2, "name": "Pattern 1", "boundary": [7, 8, 9, 10],
                           "holes": [], "internal_curves": [], "placement": {
                               "translation": [0.5, 1.0, -0.25], "rotation": [0.0, 0.1, 0.0]},
                           "material": 1, "grain_angle": 0.0}]
    undo = copy.deepcopy(seed)
    undo["project"].update(revision=2, next_id=11)
    redo = copy.deepcopy(created)
    redo["project"]["revision"] = 3
    edit = copy.deepcopy(redo)
    edit["project"]["revision"] = 4
    edit["project"]["points"][0]["position"][0] -= 0.01
    return {"created": created, "undo": undo, "redo": redo, "reopened-edit": edit}


def selfcheck():
    targets, seed = load_frozen_inputs()
    history = synthetic_history(seed)
    passed = []

    def accepts(name, action):
        action()
        passed.append({"case": name, "expected": "accept", "result": "accept"})

    def rejects(name, action):
        try:
            action()
        except (Rejected, OSError) as error:
            passed.append({"case": name, "expected": "reject", "result": "reject",
                           "reason": str(error)})
        else:
            raise AssertionError("self-check incorrectly accepted: " + name)

    accepts("synthetic translated rectangle/full history", lambda: check_history(history, seed))
    accepts("strict JSON roundtrip", lambda: check_history(decode(json.dumps(history).encode()), seed))
    translated = copy.deepcopy(history)
    for stage in ("created", "redo", "reopened-edit"):
        for point in translated[stage]["project"]["points"]:
            point["position"][0] += 1.25
            point["position"][1] -= 2.5
    accepts("independent alternate authored origin", lambda: check_history(translated, seed))
    for name, raw in {
        "duplicate top key": b'{"a":1,"a":2}',
        "duplicate nested key": b'{"a":{"x":1,"x":2}}',
        "NaN": b'{"x":NaN}', "Infinity": b'{"x":Infinity}',
        "negative Infinity": b'{"x":-Infinity}', "f64 overflow": b'{"x":1e309}',
        "integer overflow": b'{"x":18446744073709551616}',
        "invalid UTF-8": b'"\xff"', "trailing JSON": b'{}{}',
        "unpaired Unicode surrogate": b'"\\ud800"',
        "overdeep JSON": b'[' * 2000 + b']' * 2000,
    }.items():
        rejects(name, lambda raw=raw: decode(raw))
    rejects("byte cap before parsing", lambda: decode(b' ' * (MAX_BYTES + 1)))

    def altered(name, stage, path, value):
        candidate = copy.deepcopy(history)
        target = candidate[stage]
        for item in path[:-1]:
            target = target[item]
        target[path[-1]] = value
        rejects(name, lambda: check_history(candidate, seed))

    mutations = [
        ("extra envelope field", "created", ["unexpected"], None),
        ("wrong version", "created", ["version"], 4),
        ("extra project field", "created", ["project", "unexpected"], []),
        ("material changed", "created", ["project", "materials", 0, "areal_density"], 0.19),
        ("f64 changed to integer", "created", ["project", "materials", 0, "warp_stiffness"], 400),
        ("resolution changed", "created", ["project", "resolution"], 0.08),
        ("wrong created revision", "created", ["project", "revision"], 2),
        ("wrong allocator", "created", ["project", "next_id"], 12),
        ("point extra field", "created", ["project", "points", 0, "unexpected"], 0),
        ("boolean point id", "created", ["project", "points", 0, "id"], True),
        ("floating point id", "created", ["project", "points", 0, "id"], 3.0),
        ("integer coordinate", "created", ["project", "points", 0, "position", 0], 0),
        ("width wrong", "created", ["project", "points", 1, "position", 0], 0.04),
        ("geometry outside absolute tolerance", "created", ["project", "points", 1, "position", 0], 0.030000000002),
        ("height wrong", "created", ["project", "points", 2, "position", 1], 0.83),
        ("nonfinite coordinate", "created", ["project", "points", 0, "position", 0], math.inf),
        ("open loop", "created", ["project", "curves", 3, "end"], 4),
        ("curved edge", "created", ["project", "curves", 0, "controls"], [[0.0, 0.0], [0.1, 0.1]]),
        ("curve extra field", "created", ["project", "curves", 0, "unexpected"], None),
        ("wrong piece name", "created", ["project", "pieces", 0, "name"], "Pattern1"),
        ("piece extra field", "created", ["project", "pieces", 0, "unexpected"], None),
        ("wrong boundary", "created", ["project", "pieces", 0, "boundary"], [7, 8, 10, 9]),
        ("extra hole", "created", ["project", "pieces", 0, "holes"], [[7, 8, 9, 10]]),
        ("internal curve", "created", ["project", "pieces", 0, "internal_curves"], [7]),
        ("wrong material", "created", ["project", "pieces", 0, "material"], 2),
        ("wrong grain", "created", ["project", "pieces", 0, "grain_angle"], 0.1),
        ("placement extra field", "created", ["project", "pieces", 0, "placement", "unexpected"], []),
        ("short placement", "created", ["project", "pieces", 0, "placement", "translation"], [0.0, 1.0]),
        ("integer placement", "created", ["project", "pieces", 0, "placement", "rotation"], [0, 0.0, 0.0]),
        ("undo allocator reset", "undo", ["project", "next_id"], 2),
        ("undo geometry remains", "undo", ["project", "points"], [{"id": 3, "position": [0.0, 0.0]}]),
        ("undo wrong revision", "undo", ["project", "revision"], 1),
        ("redo hidden placement change", "redo", ["project", "pieces", 0, "placement", "rotation", 1], 0.2),
        ("redo signed-zero bit change", "redo", ["project", "pieces", 0, "grain_angle"], -0.0),
        ("redo numeric type change", "redo", ["project", "points", 0, "id"], 3.0),
        ("edit no movement", "reopened-edit", ["project", "points", 0, "position", 0], -0.37),
        ("edit wrong direction", "reopened-edit", ["project", "points", 0, "position", 0], -0.36),
        ("edit wrong distance", "reopened-edit", ["project", "points", 0, "position", 0], -0.39),
        ("edit other point changed", "reopened-edit", ["project", "points", 1, "position", 1], 0.23),
        ("edit placement changed", "reopened-edit", ["project", "pieces", 0, "placement", "translation", 1], 1.01),
        ("edit material changed", "reopened-edit", ["project", "materials", 0, "friction"], 0.5),
        ("edit wrong revision", "reopened-edit", ["project", "revision"], 5),
    ]
    for mutation in mutations:
        altered(*mutation)
    nonlattice = copy.deepcopy(history)
    for stage in ("created", "redo", "reopened-edit"):
        for point in nonlattice[stage]["project"]["points"]:
            point["position"][0] += 0.001
    rejects("correct dimensions at nonlattice origin", lambda: check_history(nonlattice, seed))
    reversed_loop = copy.deepcopy(history)
    for stage in ("created", "redo", "reopened-edit"):
        points = reversed_loop[stage]["project"]["points"]
        points[1]["position"], points[3]["position"] = points[3]["position"], points[1]["position"]
    rejects("clockwise/reordered geometry", lambda: check_history(reversed_loop, seed))
    accepts("analytic rectangle area", lambda: close(loop_area([[1.0, 2.0], [1.4, 2.0],
                                                                  [1.4, 2.6], [1.0, 2.6]]), 0.24, "area"))
    accepts("analytic trapezoid area", lambda: close(loop_area([[0.99, 2.0], [1.4, 2.0],
                                                                  [1.4, 2.6], [1.0, 2.6]]), 0.243, "area"))
    with tempfile.TemporaryDirectory(prefix=".synthetic-", dir=BASE) as directory:
        temporary = Path(directory)
        regular = temporary / "regular.json"
        regular.write_bytes(b'{}')
        accepts("regular file read", lambda: same(read_file(str(regular.relative_to(ROOT)))[0], b'{}'))
        link = temporary / "link.json"
        link.symlink_to(regular.name)
        rejects("file symlink refusal", lambda: read_file(str(link.relative_to(ROOT))))
        link_dir = temporary / "linked-directory"
        link_dir.symlink_to(temporary, target_is_directory=True)
        rejects("parent symlink refusal", lambda: read_file(str((link_dir / regular.name).relative_to(ROOT))))
        rejects("directory refusal", lambda: read_file(str(temporary.relative_to(ROOT))))
        regular.write_bytes(b' ' * (MAX_BYTES + 1))
        rejects("on-disk byte cap", lambda: read_file(str(regular.relative_to(ROOT))))
    rejects("parent traversal refusal", lambda: read_file(".local/../README.md"))
    rejects("absolute path refusal", lambda: read_file(str(ROOT / "README.md")))
    receipt = {"status": "PASS", "actual_case_files_checked": False,
               "seed_sha256": targets["seed"]["sha256"],
               "synthetic_history_sha256": hashlib.sha256(json.dumps(history, sort_keys=True).encode()).hexdigest(),
               "count": len(passed), "checks": passed}
    output = BASE / "selfcheck-receipt.json"
    with output.open("x", encoding="utf-8") as stream:
        json.dump(receipt, stream, indent=2, allow_nan=False)
        stream.write("\n")
    print(json.dumps({"status": "PASS", "selfchecks": len(passed), "receipt": str(output.relative_to(ROOT))}))


def run_once(manifest_relative):
    marker = BASE / "run-started.json"
    with marker.open("x", encoding="utf-8") as stream:
        json.dump({"started_utc": datetime.now(timezone.utc).isoformat(),
                   "output_hash_manifest": manifest_relative}, stream)
        stream.write("\n")
    receipt = {"status": "FAIL", "file_only": True, "actual_case_execution_count": 1,
               "limits": ["No UI, gesture, process lifecycle, native reopen, reference, physics, or full garment acceptance."]}
    before = {}
    after = {}
    watched = []
    try:
        freeze_relative = str((BASE / "freeze.json").relative_to(ROOT))
        freeze_raw, freeze_meta = read_file(freeze_relative)
        freeze = decode(freeze_raw)
        receipt["freeze_sha256"] = freeze_meta["sha256"]
        for relative, expected in freeze["artifacts"].items():
            same(read_file(relative)[1], expected, "frozen artifact " + relative)
        targets, seed = load_frozen_inputs()
        authorized_raw, authorized_meta = read_file(manifest_relative)
        authorized = decode(authorized_raw)
        require(type(authorized) is dict and authorized.keys() == targets["files"].keys(),
                "authorized hash list must cover exactly all five output files")
        for relative, digest in authorized.items():
            require(type(digest) is str and len(digest) == 64 and
                    all(char in "0123456789abcdef" for char in digest), "invalid authorized SHA-256")
        watched = list(freeze["artifacts"]) + [freeze_relative, manifest_relative, targets["seed"]["path"]] + list(targets["files"])
        watched = list(dict.fromkeys(watched))
        raws = {}
        for relative in watched:
            raws[relative], before[relative] = read_file(relative)
        same(before[manifest_relative], authorized_meta, "authorization manifest metadata")
        for relative, digest in authorized.items():
            same(before[relative]["sha256"], digest, "authorized output hash " + relative)
        documents = {stage: decode(raws[relative]) for relative, stage in targets["files"].items() if stage != "work"}
        receipt["metrics"] = check_history(documents, seed)
        by_stage = {stage: relative for relative, stage in targets["files"].items()}
        require(raws[by_stage["work"]] == raws[by_stage["reopened-edit"]], "work is not byte-equal to R4")
        receipt["semantic_status"] = "PASS"
    except Exception as error:
        receipt["error"] = type(error).__name__ + ": " + str(error)
    finally:
        for relative in watched:
            try:
                after[relative] = read_file(relative)[1]
            except Exception as error:
                after[relative] = {"read_error": type(error).__name__ + ": " + str(error)}
        unchanged = bool(watched) and len(before) == len(watched) and before == after
        receipt.update(before=before, after=after, inputs_unchanged=unchanged)
        if receipt.get("semantic_status") == "PASS" and unchanged:
            receipt["status"] = "PASS"
        elif not unchanged:
            receipt["integrity_error"] = "file hashes, byte sizes or modes changed, or an input could not be read"
        receipt["finished_utc"] = datetime.now(timezone.utc).isoformat()
        with (BASE / "run-receipt.json").open("x", encoding="utf-8") as stream:
            json.dump(receipt, stream, indent=2, allow_nan=False)
            stream.write("\n")
    print(json.dumps({"status": receipt["status"], "receipt": str((BASE / "run-receipt.json").relative_to(ROOT)),
                      "error": receipt.get("error"), "inputs_unchanged": receipt["inputs_unchanged"]}))
    return 0 if receipt["status"] == "PASS" else 1


if __name__ == "__main__":
    if sys.argv[1:] == ["--selfcheck"]:
        selfcheck()
    elif len(sys.argv) == 3 and sys.argv[1] == "--run-once":
        sys.exit(run_once(sys.argv[2]))
    else:
        raise SystemExit("usage: checker.py --selfcheck | --run-once ROOT_RELATIVE_AUTHORIZED_HASH_LIST")
