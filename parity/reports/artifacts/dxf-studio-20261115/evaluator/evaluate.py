#!/usr/bin/env python3
"""Prospective fixture-specific evaluator; never imports or invokes WeftForge."""
import argparse
import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import sys
import tempfile

import ezdxf

HERE = Path(__file__).resolve().parent
sys.dont_write_bytecode = True
module = importlib.util.spec_from_file_location("independent_svg_consumer", HERE / "svg_checker_frozen.py")
svg = importlib.util.module_from_spec(module)
module.loader.exec_module(svg)
require, exact = svg.require, svg.exact


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def frozen():
    manifest = svg.load_json(HERE / "FROZEN.json")
    for name, sha in manifest["files_sha256"].items():
        require(digest(HERE / name) == sha, f"frozen input/evaluator changed: {name}")
    require(ezdxf.__version__ == "1.4.5", "independent consumer must be pinned ezdxf1.4.5")
    return manifest


def expected():
    wanted, baseline = svg.load_json(HERE / "EXPECTED.json"), svg.load_json(HERE / "baseline.weftforge")
    exact(baseline["project"]["revision"], wanted["baseline_revision"], "original baseline revision")
    exact(baseline["project"]["next_id"], wanted["baseline_next_id"], "original baseline allocator")
    require(wanted["after_import_next_id"] == 11 + 2 * (1 + 4 + 4), "analytic object allocation changed")
    source, _ = read_dxf(HERE / "original-editable-patterns.dxf")
    svg.compare_geometry(source, wanted["source_geometry"], "independently consumed original DXF")
    return wanted, baseline


def raw_layers(path):
    lines = Path(path).read_text(encoding="ascii").splitlines()
    require(len(lines) % 2 == 0, "DXF has incomplete group/value pairs")
    records = []
    for i in range(0, len(lines), 2):
        code, value = int(lines[i].strip()), lines[i + 1].strip()
        if code == 0:
            records.append([])
        require(records, "DXF group outside record")
        records[-1].append((code, value))
    result = []
    for record in records:
        if record[0] == (0, "LAYER"):
            names = [value for code, value in record if code == 2]
            require(len(names) == 1, "DXF layer record needs one name")
            result.append(names[0])
    return result


def read_dxf(path):
    before = digest(path)
    drawing = ezdxf.readfile(path)
    require(drawing.dxfversion == "AC1015", "DXF version is not R2000")
    require(drawing.header["$INSUNITS"] == 4, "DXF physical units are not millimetres")
    require(drawing.header["$MEASUREMENT"] == 1 and drawing.header["$LUNITS"] == 2, "DXF metric/decimal units changed")
    audit = drawing.audit()
    require(not audit.errors and not audit.fixes, "ordinary DXF consumer required repair or reported an error")
    paths = []
    for entity in drawing.modelspace():
        layer = entity.dxf.layer
        if not paths or paths[-1][0] != layer:
            require(layer not in [name for name, _ in paths], "DXF piece entities interleaved")
            paths.append((layer, []))
        require(entity.dxf.get("paperspace", 0) == 0, "paper-space boundary entity")
        if entity.dxftype() == "LINE":
            points = [tuple(entity.dxf.start), tuple(entity.dxf.end)]
            require(entity.dxf.get("thickness", 0.0) == 0.0, "DXF line thickness is not zero")
            controls = None
        else:
            require(entity.dxftype() == "SPLINE", "DXF boundary sampled/replaced by unsupported entity")
            require(entity.dxf.flags == 8 and entity.dxf.degree == 3, "DXF cubic flags/degree changed")
            require(list(entity.knots) == [0.0] * 4 + [1.0] * 4, "DXF clamped cubic knots changed")
            require(len(entity.control_points) == 4 and not len(entity.fit_points) and not len(entity.weights), "DXF cubic is not nonrational four-control construction")
            require(tuple(entity.dxf.extrusion) == (0.0, 0.0, 1.0), "DXF spline normal changed")
            points = [tuple(float(v) for v in point) for point in entity.control_points]
            controls = tuple((p[0] / 1000.0, p[1] / 1000.0) for p in points[1:3])
            curve = entity.construction_tool()
            for k in range(17):
                t = k / 16
                actual = tuple(float(v) / 1000.0 for v in tuple(curve.point(t))[:2])
                span = (tuple(v / 1000.0 for v in points[0][:2]), controls,
                        tuple(v / 1000.0 for v in points[-1][:2]))
                svg.near(actual, svg.sample(span, t), svg.SAMPLE_TOLERANCE, "independent B-spline/Bernstein sample")
        require(all(len(point) == 3 and point[2] == 0.0 for point in points), "DXF Z is not zero")
        paths[-1][1].append((tuple(v / 1000.0 for v in points[0][:2]), controls,
                            tuple(v / 1000.0 for v in points[-1][:2])))
    require(paths, "DXF has no model-space boundary")
    require(raw_layers(path) == ["0"] + [name for name, _ in paths], "DXF layer declaration order/content changed")
    require({block.name for block in drawing.blocks} == {"*Model_Space", "*Paper_Space"}, "DXF includes an extra named block")
    for name, spans in paths:
        require(name.startswith("WF_PIECE_") and name in drawing.layers, "DXF piece layer missing")
        require(len(spans) >= 3 and all(a[2] == b[0] for a, b in zip(spans, spans[1:] + spans[:1])), "DXF boundary is not exactly ordered/closed")
    require(not len(drawing.paperspace()), "DXF includes paper-space entities")
    require(digest(path) == before, "independent consumer modified its input")
    return paths, {"consumer": "ezdxf", "version": ezdxf.__version__, "ordinary_readfile": True,
                   "audit_errors": 0, "audit_fixes": 0, "input_sha256": before}


def old_document(baseline, pending):
    old = copy.deepcopy(baseline)
    if pending:
        for point in old["project"]["points"]:
            if point["position"][0] == 0.6:
                point["position"][0] = 0.625
    return old


def geometry(baseline, wanted, pending=False, edited=False):
    result = svg.native_geometry(old_document(baseline, pending)["project"]) + copy.deepcopy(wanted["source_geometry"])
    if edited:
        result[1][1][1][1][0][0] = wanted["native_cubic_edit"]["after_m"]
    return result


def check_native(document, baseline, wanted, pending=False, edited=False, revision=2):
    old = old_document(baseline, pending)
    if pending:
        for i in (1, 2):
            old["project"]["points"][i]["position"][0] = document["project"]["points"][i]["position"][0]
    svg.check_native(document, baseline, old, geometry(baseline, wanted, pending, edited))
    exact(document["project"]["next_id"], wanted["after_import_next_id"], "monotonic stable-ID high-water")
    exact(document["project"]["revision"], revision, "authoring revision")


def changed_revision(document, revision):
    result = copy.deepcopy(document)
    result["project"]["revision"] = revision
    return result


def cubic(document):
    piece = document["project"]["pieces"][1]
    require(piece["name"] == "WF_PIECE_17", "wrong first imported piece")
    return svg.id_map(document["project"]["curves"], "curves")[piece["boundary"][1]]


def check_rectangle(document, baseline, width, revision, next_id):
    wanted = changed_revision(baseline, revision)
    wanted["project"]["next_id"] = next_id
    for i in (1, 2):
        actual = document["project"]["points"][i]["position"]
        svg.near(actual, [0.5 + width, baseline["project"]["points"][i]["position"][1]], svg.POINT_TOLERANCE, "independently predicted rectangle corner")
        wanted["project"]["points"][i]["position"][0] = actual[0]
    exact(document, wanted, "only declared rectangle width/revision/allocator may differ")


def check_history(documents, baseline, wanted):
    for name, revision in wanted["native_revisions"].items():
        exact(documents[name]["project"]["revision"], revision, name + " revision")
    check_rectangle(documents["preimport"], baseline, 0.11, 4, 11)
    check_native(documents["imported"], baseline, wanted, pending=True, revision=5)
    undo = changed_revision(documents["preimport"], 6)
    undo["project"]["next_id"] = 29
    exact(documents["undo_import"], undo, "one Undo restores preimport authoring; high-water retained")
    prior = changed_revision(baseline, 7)
    prior["project"]["next_id"] = 29
    exact(documents["undo_prior"], prior, "prior width command remains undoable after import")
    redo_prior = changed_revision(documents["preimport"], 8)
    redo_prior["project"]["next_id"] = 29
    exact(documents["redo_prior"], redo_prior, "prior width command redoes exactly")
    exact(documents["redo_import"], changed_revision(documents["imported"], 9), "import Redo restores all stable IDs and authoring")
    edited = changed_revision(documents["imported"], 10)
    cubic(edited)["controls"][0][0] = wanted["native_cubic_edit"]["after_m"]
    exact(documents["edited"], edited, "one editable cubic control change")
    check_native(documents["edited"], baseline, wanted, pending=True, edited=True, revision=10)
    for name in ("rejected", "reopened"):
        exact(documents[name], documents["edited"], name + " preserves every authored field and revision")


def check_exports(document, exported_svg, exported_dxf):
    wanted = svg.native_geometry(document["project"])
    exported, svg_scale = svg.read_svg(Path(exported_svg).read_bytes())
    svg.compare_geometry(exported, wanted, "independent exported SVG")
    by_name = dict(wanted)
    dxf_wanted = [("WF_PIECE_" + str(p["id"]), by_name[p["name"]])
                  for p in sorted(document["project"]["pieces"], key=lambda p: p["id"])]
    exported, consumer = read_dxf(exported_dxf)
    svg.compare_geometry(exported, dxf_wanted, "independent exported DXF")
    return {"svg_scale": svg_scale, "dxf_consumer": consumer}


def synthetic_import(baseline, wanted, pending):
    old = old_document(baseline, pending)
    document = svg.synthetic_native(baseline, old, geometry(baseline, wanted, pending))
    project = document["project"]
    fresh = [obj["id"] for key in ("points", "curves", "pieces") for obj in project[key] if obj["id"] >= 11]
    mapping = dict(zip(fresh, reversed(range(11, 29))))
    for key in ("points", "curves", "pieces"):
        for obj in project[key]:
            obj["id"] = mapping.get(obj["id"], obj["id"])
            if key == "curves":
                for field in ("start", "end"):
                    obj[field] = mapping.get(obj[field], obj[field])
            if key == "pieces":
                obj["boundary"] = [mapping.get(i, i) for i in obj["boundary"]]
    project["next_id"], project["revision"] = 29, 2
    return document


def synthetic_dxf(path, document):
    drawing = ezdxf.new("R2000")
    drawing.layers.remove("Defpoints")
    drawing.header["$INSUNITS"], drawing.header["$MEASUREMENT"], drawing.header["$LUNITS"] = 4, 1, 2
    by_name = dict(svg.native_geometry(document["project"]))
    for piece in sorted(document["project"]["pieces"], key=lambda p: p["id"]):
        layer = "WF_PIECE_" + str(piece["id"])
        drawing.layers.new(layer)
        for a, controls, b in by_name[piece["name"]]:
            millimetres = lambda p: tuple(v * 1000.0 for v in p) + (0.0,)
            if controls is None:
                drawing.modelspace().add_line(millimetres(a), millimetres(b), dxfattribs={"layer": layer})
            else:
                entity = drawing.modelspace().add_spline(dxfattribs={"layer": layer, "flags": 8, "extrusion": (0, 0, 1)})
                entity.control_points = [millimetres(p) for p in (a, *controls, b)]
                entity.knots = [0.0] * 4 + [1.0] * 4
    drawing.saveas(path)


def self_check(baseline, wanted):
    good = synthetic_import(baseline, wanted, True)
    good["project"]["revision"] = 5
    preimport = changed_revision(baseline, 4)
    for i in (1, 2):
        preimport["project"]["points"][i]["position"][0] = 0.61
    undo = changed_revision(preimport, 6)
    undo["project"]["next_id"] = 29
    prior = changed_revision(baseline, 7)
    prior["project"]["next_id"] = 29
    redo_prior = changed_revision(preimport, 8)
    redo_prior["project"]["next_id"] = 29
    edited = changed_revision(good, 10)
    cubic(edited)["controls"][0][0] = 0.0
    history = {"preimport": preimport, "imported": good, "undo_import": undo, "undo_prior": prior,
               "redo_prior": redo_prior, "redo_import": changed_revision(good, 9),
               "edited": edited, "rejected": copy.deepcopy(edited), "reopened": copy.deepcopy(edited)}
    check_history(history, baseline, wanted)
    check_native(synthetic_import(baseline, wanted, False), baseline, wanted)
    rejected = []

    def reject(label, action):
        try:
            action()
        except (OSError, ValueError, KeyError, TypeError, IndexError):
            rejected.append(label)
        else:
            raise ValueError("self-check accepted " + label)

    def mutate(name, action):
        wrong = copy.deepcopy(history)
        action(wrong[name]["project"])
        check_history(wrong, baseline, wanted)

    reject("wrong import scale", lambda: mutate("imported", lambda p: p["points"][4]["position"].__setitem__(0, -250.0)))
    reject("wrong origin/Y inversion", lambda: mutate("imported", lambda p: p["points"][4]["position"].__setitem__(1, -0.15)))
    reject("lost asymmetric cubic control", lambda: mutate("imported", lambda p: p["curves"][5]["controls"][0].__setitem__(0, -0.005)))
    reject("layer names reordered", lambda: mutate("imported", lambda p: p["pieces"].__setitem__(slice(1, 3), p["pieces"][1:3][::-1])))
    reject("nondefault imported placement", lambda: mutate("imported", lambda p: p["pieces"][1]["placement"]["translation"].__setitem__(0, 0.01)))
    reject("Undo rolls allocator back", lambda: mutate("undo_import", lambda p: p.__setitem__("next_id", 11)))
    reject("prior Undo retains wrong width", lambda: mutate("undo_prior", lambda p: p["points"][1]["position"].__setitem__(0, 0.61)))
    reject("Redo changes stable ID", lambda: mutate("redo_import", lambda p: p["pieces"][1].__setitem__("id", 100)))
    reject("rejection mutates old material", lambda: mutate("rejected", lambda p: p["materials"][0].__setitem__("friction", 0.3)))
    reject("reopen drops cubic edit", lambda: mutate("reopened", lambda p: p["curves"][5]["controls"][0].__setitem__(0, -0.01)))
    with tempfile.TemporaryDirectory(prefix="weftforge-dxf-independent-") as temporary:
        directory = Path(temporary)
        dxf, exported_svg = directory / "synthetic.dxf", directory / "synthetic.svg"
        synthetic_dxf(dxf, edited)
        exported_svg.write_bytes(svg.synthetic_svg(svg.native_geometry(edited["project"])))
        check_exports(edited, exported_svg, dxf)
        exported_svg.write_bytes(exported_svg.read_bytes().replace(b'width="1000mm"', b'width="100mm"'))
        reject("wrong SVG physical scale", lambda: check_exports(edited, exported_svg, dxf))
        exported_svg.write_bytes(svg.synthetic_svg(svg.native_geometry(edited["project"])))
        drawing = ezdxf.readfile(dxf)
        drawing.header["$INSUNITS"] = 1
        drawing.saveas(dxf)
        reject("wrong DXF physical units", lambda: check_exports(edited, exported_svg, dxf))
        reject("missing actual artifact", lambda: svg.load_json(directory / "missing.weftforge"))
    return {"status": "SYNTHETIC_SELF_CHECKS_PASSED", "product_execution": "NOT_RUN", "permuted_fresh_ids": True, "rejected": rejected}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-check", action="store_true")
    parser.add_argument("--native-checkpoints", type=Path, help="directory containing the nine declared native checkpoint filenames")
    parser.add_argument("--imported", type=Path, help="CLI/MCP/shared-command native output; no pending native edit")
    parser.add_argument("--exported-svg", type=Path)
    parser.add_argument("--exported-dxf", type=Path)
    args = parser.parse_args()
    manifest = frozen()
    wanted, baseline = expected()
    if args.self_check:
        require(not any((args.native_checkpoints, args.imported, args.exported_svg, args.exported_dxf)), "self-check cannot grade product artifacts")
        return self_check(baseline, wanted)
    require(bool(args.native_checkpoints) != bool(args.imported), "choose one actual native workflow or headless imported artifact")
    require(args.exported_svg and args.exported_dxf, "both direct exported file artifacts are required")
    paths = [args.exported_svg, args.exported_dxf]
    if args.native_checkpoints:
        checkpoint_paths = {name: args.native_checkpoints / (name + ".weftforge") for name in wanted["native_revisions"]}
        documents = {name: svg.load_json(path) for name, path in checkpoint_paths.items()}
        check_history(documents, baseline, wanted)
        document = documents["reopened"]
        paths += list(checkpoint_paths.values())
    else:
        document = svg.load_json(args.imported)
        check_native(document, baseline, wanted)
        paths.append(args.imported)
    require(len({p.resolve() for p in paths}) == len(paths), "actual artifacts must be distinct paths")
    require(not any(p.resolve().is_relative_to(HERE) for p in paths), "actual product artifacts cannot replace frozen evaluator inputs")
    exports = check_exports(document, args.exported_svg, args.exported_dxf)
    frozen()
    return {"status": "INDEPENDENT_STATIC_CHECKS_PASSED", "frozen_utc": manifest["frozen_utc"],
            "artifact_sha256": {str(path): digest(path) for path in paths}, "exports": exports,
            "native_actions": "REQUIRE_SEPARATE_RAW_TRACE_REVIEW", "reference_parity": "NOT_ESTABLISHED",
            "full_DXF_profile": "PARTIAL", "all_save_failure_recovery_retention": "NOT_CLAIMED"}


if __name__ == "__main__":
    try:
        result = main()
    except (OSError, ValueError, KeyError, TypeError, IndexError, ezdxf.DXFError) as error:
        print(json.dumps({"status": "FAILED", "error": str(error), "native_acceptance": "NOT_ESTABLISHED"}, indent=2))
        sys.exit(1)
    print(json.dumps(result, indent=2))
