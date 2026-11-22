"""Compare the retained native and CLI exports with their source OBJ triangles."""
from collections import defaultdict
import json
import math
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
ROOT = Path(sys.argv[1]) if len(sys.argv) == 2 else Path.cwd()


def read_obj(path):
    xyz, uv, panels, colors = [], [], defaultdict(list), {}
    group, material = None, None
    for line in path.read_text().splitlines():
        fields = line.split()
        if not fields:
            continue
        kind, data = fields[0], fields[1:]
        if kind == "v":
            xyz.append(tuple(map(float, data)))
        elif kind == "vt":
            uv.append(tuple(map(float, data)))
        elif kind in {"o", "g"}:
            group = " ".join(data)
        elif kind == "mtllib":
            library = " ".join(data)
            assert Path(library).name == library
            for entry in path.with_name(library).read_text().splitlines():
                tokens = entry.split()
                if not tokens:
                    continue
                if tokens[0] == "newmtl":
                    name = " ".join(tokens[1:])
                elif tokens[0] == "Kd":
                    colors[name] = tuple(map(float, tokens[1:]))
        elif kind == "usemtl":
            material = " ".join(data)
        elif kind == "f":
            assert group and len(data) == 3
            face = []
            for corner in data:
                point, tex = map(int, corner.split("/")[:2])
                assert 0 < point <= len(xyz) and 0 < tex <= len(uv)
                face.append((xyz[point - 1], uv[tex - 1]))
            panels[(group, material)].append(face)
    return [(colors.get(mat), faces) for (_, mat), faces in panels.items()]


def extents(points):
    return tuple(max(axis) - min(axis) for axis in zip(*points))


def area(face, column):
    a, b, c = (vertex[column] for vertex in face)
    u = [x - y for x, y in zip(b, a)]
    v = [x - y for x, y in zip(c, a)]
    if len(u) == 2:
        return abs(u[0] * v[1] - u[1] * v[0]) / 2
    return math.hypot(*(u[(i+1)%3]*v[(i+2)%3] - u[(i+2)%3]*v[(i+1)%3] for i in range(3))) / 2


def oriented(faces):
    triangles = []
    for face in faces:
        triplet = tuple(point for point, _ in face)
        triangles.append(min(triplet[i:] + triplet[:i] for i in range(3)))
    return sorted(triangles)


def compare(source, output):
    before, after = read_obj(source), read_obj(output)
    assert len(before) == len(after) and before
    result = []
    for color, original in before:
        matches = [faces for rgb, faces in after if color is None or max(abs(c-d) for c,d in zip(color, rgb, strict=True)) <= 1e-6]
        assert len(matches) == 1
        actual = matches[0]
        expected_xyz, actual_xyz = oriented(original), oriented(actual)
        assert len(expected_xyz) == len(actual_xyz)
        error = max(abs(x-y) for a,b in zip(expected_xyz,actual_xyz)
                    for p,q in zip(a,b) for x,y in zip(p,q))
        assert error <= 1e-9
        scale = math.sqrt(sum(area(face,0) for face in original) / sum(area(face,1) for face in original))
        input_size = extents(uv for face in original for _,uv in face)
        size = extents(uv for face in actual for _,uv in face)
        assert all(abs(a - b*scale) <= 1e-9 for a,b in zip(size,input_size))
        result.append({"triangles":len(actual),"max_component_error_m":error,"chart_size_m":size,
                       "source_diffuse_rgb":color,"metres_per_uv_unit":scale})
    return result


def main():
    source = ROOT / "fixtures/posed-obj"
    bent = compare(source / "Bent swatch.obj", HERE / "Bent reopened.obj")
    assert (HERE / "Bent reopened.obj").read_bytes() == (HERE / "Bent restored.obj").read_bytes()
    assert all(abs(x-y) <= 1e-9 for x,y in zip(bent[0]["chart_size_m"],(.24,.36)))
    projects = [json.loads((HERE / name).read_bytes()) for name in ("Bent native.weftforge","Bent restored.weftforge")]
    assert all(p["version"] == 8 for p in projects)
    assert [p["project"].pop("revision") for p in projects] == [3,5]
    assert projects[0] == projects[1]
    cleared = read_obj(HERE / "Bent cleared.obj")
    vertices = {p for face in cleared[0][1] for p,_ in face}
    assert len(cleared) == 1 and len(vertices) == 49 and extents(vertices)[2] == 0.
    model_io = json.loads((HERE / "Bent ModelIO.json").read_bytes())
    assert len(model_io) == 1 and model_io[0]["indices"] == 12 and model_io[0]["vertices"] == 6
    print(json.dumps({"native_bent":bent,"native_cleared_flat_vertices":len(vertices),
                      "restored_export_byte_identical":True,"independent_modelio":model_io,
                      "cli_extended":compare(source / "Extended bent swatch.obj",HERE / "Extended initial.obj"),
                      "cli_harbor":compare(ROOT / "parity/reports/artifacts/native-selected-obj-20261120/Harbor draped.obj",
                                           HERE / "Harbor initial.obj")},indent=2))


if __name__ == "__main__":
    main()
