"""Compare files exported through the native timeline with the accepted solver steps."""
import base64
import json
import math
from pathlib import Path
import struct

HERE = Path(__file__).resolve().parent


def obj(path):
    points = []
    faces = []
    for line in path.read_text().splitlines():
        fields = line.split()
        if fields[:1] == ["v"]:
            points.append(tuple(map(float, fields[1:4])))
        if fields[:1] == ["f"]:
            faces.append(tuple(int(field.split("/")[0]) - 1 for field in fields[1:]))
    assert len(points) == 6 and len(faces) == 4
    return points, faces


def max_error(a, b):
    return max(abs(x - y) for p, q in zip(a, b, strict=True) for x, y in zip(p, q, strict=True))


def gltf(path):
    scene = json.loads(path.read_bytes())
    (prim,) = scene["meshes"][0]["primitives"]
    acc = scene["accessors"][prim["attributes"]["POSITION"]]
    assert acc["componentType"] == 5126 and acc["type"] == "VEC3"
    view = scene["bufferViews"][acc["bufferView"]]
    uri = scene["buffers"][view["buffer"]]["uri"]
    data = base64.b64decode(uri.split(",", 1)[1], validate=True)
    start = view.get("byteOffset", 0) + acc.get("byteOffset", 0)
    stride = view.get("byteStride", 12)
    points = [struct.unpack_from("<3f", data, start + i * stride) for i in range(acc["count"])]
    return points, scene["materials"][prim["material"]]["pbrMetallicRoughness"]


def main():
    results = {}
    for name, reference in [
        ("Native initial", "CLI initial"),
        ("Native first accepted", "CLI step 1"),
        ("Native second accepted", "CLI step 2"),
        ("Native step 4", "CLI step 4"),
    ]:
        exact, faces = obj(HERE / (reference + ".obj"))
        actual, native_faces = obj(HERE / (name + ".obj"))
        error = max_error(actual, exact)
        assert native_faces == faces and error <= 1e-9, (name, error)
        results[name] = error
    first, _ = obj(HERE / "CLI step 1.obj")
    last, _ = obj(HERE / "CLI step 4.obj")
    assert max_error(first, last) > 1e-6
    display, material = gltf(HERE / "Scrubbed finish.gltf")
    assert max_error(first, display) <= 1e-6
    assert math.isclose(material["metallicFactor"], 0.5, abs_tol=1e-6)
    before_file = json.loads((HERE / "Swatch.weftforge").read_bytes())
    after_file = json.loads((HERE / "Resized recorded swatch.weftforge").read_bytes())
    before, after = before_file["project"], after_file["project"]
    assert before_file["version"] == after_file["version"] == 8
    assert (before["revision"], after["revision"]) == (3, 5)
    assert after["materials"][0]["surface"] == {"metallic": 0.5, "roughness": 1.0}
    heights = []
    for project, expected in [(before, 0.36), (after, 0.37)]:
        ys = [point["position"][1] for point in project["points"]]
        height = max(ys) - min(ys)
        assert math.isclose(height, expected, abs_tol=1e-9)
        heights.append(height)
    results = {
        "saved_pattern_heights_m": heights,
        "native_xyz_error_m": results,
        "gltf_xyz_error_m": max_error(first, display),
        "metallic": material["metallicFactor"],
        "step_1_to_4_max_displacement_component_m": max_error(first, last),
    }
    print(json.dumps(results, indent=2))


if __name__ == "__main__":
    main()
