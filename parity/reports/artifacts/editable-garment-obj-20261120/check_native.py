"""Compare the saved native cases with their known source surfaces and a release CLI."""

from collections import Counter, defaultdict
from copy import deepcopy
from hashlib import sha256
import json
from math import dist, hypot, isclose
from pathlib import Path
import struct
import subprocess
import sys
from tempfile import TemporaryDirectory

evidence = Path(__file__).resolve().parent
capture = evidence / "macos-editing"
artifacts = evidence.parent
source = artifacts / "native-obj-fabrics-20261120"
sys.path.insert(0, str(source))
sys.dont_write_bytecode = True
from check_fabrics import read_surface

inputs = evidence / "inputs"
cli = Path(sys.argv[1]).resolve()


def digest(p):
    return sha256(p.read_bytes()).hexdigest()


def project(p):
    return json.loads(p.read_bytes())["project"]


def one_surface(p):
    [surface] = read_surface(p)["materials"].values()
    return surface


def boundary_vertices(path):
    vertices = []
    edges = defaultdict(Counter)
    for line in path.read_text().splitlines():
        row = line.split()
        if not row:
            continue
        if row[0] == "g":
            group = row[1]
        elif row[0] == "v":
            vertices.append(tuple(map(float, row[1:])))
        elif row[0] == "f":
            indices = [int(c.split("/")[0]) - 1 for c in row[1:]]
            edges[group].update(
                tuple(sorted(e)) for e in zip(indices, indices[1:] + indices[:1])
            )
    return {
        g: {vertices[i] for e, n in counts.items() if n == 1 for i in e}
        for g, counts in edges.items()
    }


def check_close(a, b):
    assert isclose(a, b, abs_tol=1e-10, rel_tol=0), (a, b)


native = project(capture / "Pocket restored.weftforge")
edited = project(capture / "Pocket edited corner.weftforge")
redo = project(capture / "Pocket edited redo.weftforge")
after_refusal = capture / "Pocket after refusal.weftforge"
assert (
    after_refusal.read_bytes() == (capture / "Pocket restored.weftforge").read_bytes()
)
assert [d["revision"] for d in (native, edited, redo)] == [3, 4, 6]
assert edited == dict(redo, revision=4)
expect = deepcopy(native)
point = next(p for p in expect["points"] if p["id"] == 14)
assert point["position"] == [0.08, 0.24]
point["position"][0] = 0.06
expect["revision"] = 4
assert expect == edited
before = one_surface(capture / "Pocket native.obj")
after = one_surface(capture / "Pocket edited corner.obj")
check_close(before["area_m2"], 0.0756)
check_close(before["boundary_m"], 1.74)
check_close(after["area_m2"], 0.0756 - 0.02 * 0.12 / 2)
check_close(after["boundary_m"], 1.74 + 0.02 + hypot(0.02, 0.12) - 0.12)
dimensions = [max(x) - min(x) for x in zip(*(p["position"] for p in native["points"]))]
assert dimensions == [0.24, 0.36] and len(native["pieces"][0]["holes"]) == 2

harbor = project(capture / "Harbor imported.weftforge")
assert len(harbor["pieces"]) == 2 and not harbor["seams"] and harbor["revision"] == 1
source_harbor = read_surface(source / "Harbor.obj")
export_harbor = read_surface(capture / "Harbor native.obj")
src_boundary = boundary_vertices(source / "Harbor.obj")
dst_boundary = boundary_vertices(capture / "Harbor native.obj")
panels = []
for piece in harbor["pieces"]:
    material = next(m for m in harbor["materials"] if m["id"] == piece["material"])
    original = source_harbor["materials"][material["name"]]
    assert (
        original["groups"] == [piece["name"]]
        and original["diffuse_rgb"] == material["color"]
    )
    remeshed = next(
        g
        for g in export_harbor["materials"].values()
        if g["groups"] == [f"piece_{piece['id']}"]
    )
    for field in ["diffuse_rgb", "triangles"]:
        assert original[field] == remeshed[field]
    for field in ["area_m2", "boundary_m"]:
        check_close(original[field], remeshed[field])
    a, b = src_boundary[piece["name"]], dst_boundary[remeshed["groups"][0]]
    assert len(a) == len(b) == 66
    distances = [
        max(min(dist(i, j) for j in y) for i in x) for x, y in ((a, b), (b, a))
    ]
    assert max(distances) < 1e-12
    panels.append(
        dict(
            obj_group=piece["name"],
            assigned_mtl_name=material["name"],
            imported_piece_id=piece["id"],
            placement=piece["placement"],
            rgb=original["diffuse_rgb"],
            boundary_vertices=len(a),
            xyz_boundary_vertex_bidirectional_max_m=max(distances),
            area_m2=remeshed["area_m2"],
            boundary_m=remeshed["boundary_m"],
            triangles=remeshed["triangles"],
        )
    )
with TemporaryDirectory() as scratch:
    scratch = Path(scratch)

    def run(*args):
        subprocess.run([cli, *args], cwd=scratch, check=True, capture_output=True)

    run("new", "empty.weftforge")
    run(
        "import-obj",
        "empty.weftforge",
        "pocket.weftforge",
        inputs / "Pocket in millimetres.obj",
        "0.001",
    )
    assert native == dict(project(scratch / "pocket.weftforge"), revision=3)
    run("export-obj", "pocket.weftforge", "pocket.obj")
    assert (capture / "Pocket native.obj").read_bytes() == (
        scratch / "pocket.obj"
    ).read_bytes()
    commands = scratch / "move.json"
    commands.write_text(
        json.dumps(
            [{"type": "move_point", "data": {"point": 14, "position": [0.06, 0.24]}}]
        )
    )
    run("execute", capture / "Pocket restored.weftforge", "edited.weftforge", commands)
    assert (capture / "Pocket edited corner.weftforge").read_bytes() == (
        scratch / "edited.weftforge"
    ).read_bytes()
    run("export-obj", "edited.weftforge", "edited.obj")
    assert (capture / "Pocket edited corner.obj").read_bytes() == (
        scratch / "edited.obj"
    ).read_bytes()
    run("import-obj", "empty.weftforge", "harbor.weftforge", source / "Harbor.obj", "1")
    assert (capture / "Harbor imported.weftforge").read_bytes() == (
        scratch / "harbor.weftforge"
    ).read_bytes()
    run("export-obj", "harbor.weftforge", "harbor.obj")
    assert (capture / "Harbor native.obj").read_bytes() == (
        scratch / "harbor.obj"
    ).read_bytes()
    for mtl in scratch.glob("*.mtl"):
        assert mtl.read_bytes() == (capture / mtl.name).read_bytes()
    comparisons = {
        "release_cli_sha256": digest(cli),
        "pocket_cli_revision_1_sha256": digest(scratch / "pocket.weftforge"),
        "pocket_native_revision_3_identical_except_revision": True,
        "pocket_export_byte_equal": True,
        "edited_project_and_export_byte_equal": True,
        "harbor_project_and_export_byte_equal": True,
        "all_exported_mtl_byte_equal": True,
    }
files = sorted(
    p for p in capture.iterdir() if p.suffix in {".obj", ".weftforge", ".png", ".mtl"}
)
images = {
    p.name: list(struct.unpack(">II", p.read_bytes()[16:24]))
    for p in files
    if p.suffix == ".png"
}
assert all(size == [1640, 1000] for size in images.values()) and len(images) == 3
result = dict(
    gui_source_commit="087123cfcd0ea48001716006bcb8f32ce8aa2fa8",
    gui_sha256="06feefb73450f24deca6c4626d6e52f409624af234bd16079981d61484dc9f21",
    snapshot_file_sha256={p.name: digest(p) for p in files},
    input_sha256={
        p.name: digest(p)
        for p in (
            source / "Harbor.obj",
            artifacts / "native-projected-faces-20261120/draped.obj",
            inputs / "Pocket in millimetres.obj",
        )
    },
    pocket=dict(
        dimensions_m=dimensions,
        holes=2,
        restored=before,
        edited=after,
        retained_after_cancel_byte_equal=True,
        edited_and_redone_equal_except_revision=True,
    ),
    harbor=panels,
    cli=comparisons,
    workspace_png_dimensions=images,
)
frozen = json.loads((capture / "NATIVE-MEASUREMENTS.json").read_text())
for key in (
    "snapshot_file_sha256",
    "input_sha256",
    "pocket",
    "harbor",
    "workspace_png_dimensions",
):
    assert result[key] == frozen[key], key
combined = evidence / "macos-combined"
receipt = json.loads((combined / "NATIVE.json").read_text())
for row in receipt["source_files"] + receipt["result_files"]:
    folder = inputs if row in receipt["source_files"] else combined
    assert Path(row["file"]).name == row["file"]
    assert digest(folder / row["file"]) == row["sha256"]

from decimal import Decimal

original = artifacts / "debian-wayland-multiple-cutouts-20261120" / "Imported.obj"
original_lines = original.read_text().splitlines()
mm_lines = (inputs / "Pocket in millimetres.obj").read_text().splitlines()
assert len(original_lines) == len(mm_lines)
for source_line, millimetre_line in zip(original_lines, mm_lines):
    if source_line.startswith("v "):
        assert [Decimal(v) * 1000 for v in source_line.split()[1:]] == [
            Decimal(v) for v in millimetre_line.split()[1:]
        ]
    else:
        assert source_line == millimetre_line
assert (
    digest(combined / "Combined pocket.weftforge")
    == comparisons["pocket_cli_revision_1_sha256"]
)
assert (combined / "Combined pocket.obj").read_bytes() == (
    capture / "Pocket native.obj"
).read_bytes()
published_material = (
    (combined / "Combined pocket.obj")
    .read_text()
    .split("mtllib ", 1)[1]
    .splitlines()[0]
)
assert (combined / published_material).read_bytes() == (
    capture / published_material
).read_bytes()
result["combined_native_commit"] = receipt["source_commit"]
result["combined_native_project_matches_cli"] = True
print(json.dumps(result, indent=2))
