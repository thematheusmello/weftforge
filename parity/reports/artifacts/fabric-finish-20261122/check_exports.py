"""Check both garment materials and the geometry buffer written by the CLI."""
import base64
from hashlib import sha256
import json
from pathlib import Path
import struct
import subprocess
import sys
from tempfile import TemporaryDirectory

ROOT = Path(__file__).resolve().parents[4]
FIXTURE = ROOT / "parity/reports/artifacts/native-obj-fabrics-20261120/Harbor two fabrics.weftforge"


def read_scene(path):
    if path.suffix == ".gltf":
        scene = json.loads(path.read_bytes())
        uri = scene["buffers"][0].pop("uri")
        assert uri.startswith("data:application/octet-stream;base64,")
        data = base64.b64decode(uri.partition(",")[2], validate=True)
    else:
        raw = path.read_bytes()
        magic, version, total, size, kind = struct.unpack_from("<4sIII4s", raw)
        assert (magic, version, total, kind) == (b"glTF", 2, len(raw), b"JSON")
        scene = json.loads(raw[20:20 + size])
        length, kind = struct.unpack_from("<I4s", raw, 20 + size)
        assert kind == b"BIN\0" and 28 + size + length == total
        data = raw[28 + size:][:scene["buffers"][0]["byteLength"]]
    assert len(data) == scene["buffers"][0]["byteLength"]
    return scene, data


def check(binary):
    original = FIXTURE.read_bytes()
    source = json.loads(original)
    with TemporaryDirectory() as directory:
        directory = Path(directory)
        def run(*args):
            subprocess.run([binary, *map(str, args)], check=True, capture_output=True)

        finished = directory / "finish.weftforge"
        run("fabric-surface", FIXTURE, finished, 68, 0.625, 0.25)
        saved = json.loads(finished.read_bytes())
        actual = saved["project"]
        assert saved["version"] == 7 and actual["revision"] == source["project"]["revision"] + 1
        actual["revision"] -= 1
        back = next(m for m in actual["materials"] if m["id"] == 68)
        assert back.pop("surface") == {"metallic": 0.625, "roughness": 0.25}
        assert actual == source["project"]

        buffers = {}
        positions = {}
        for stage in ("rest", "step"):
            for label, garment in [("matte", FIXTURE), ("finished", finished)]:
                containers = []
                for ext in ("gltf", "glb"):
                    path = directory / f"{stage}-{label}.{ext}"
                    if stage == "rest":
                        run(f"export-{ext}", garment, path)
                    else:
                        run("simulate", garment, path, 1)
                    containers.append(read_scene(path))
                assert containers[0] == containers[1], (stage, label)
                scene, data = containers[0]
                materials = {}
                for m in scene["materials"]:
                    pbr = m["pbrMetallicRoughness"]
                    materials[m["extras"]["weftforgeMaterialId"]] = (pbr["metallicFactor"], pbr["roughnessFactor"])
                assert materials == {"1": (0, 1), "68": (0.625, 0.25) if label == "finished" else (0, 1)}
                buffers[stage, label] = data
                for mesh in scene["meshes"]:
                    primitive, = mesh["primitives"]
                    accessor = scene["accessors"][primitive["attributes"]["POSITION"]]
                    view = scene["bufferViews"][accessor["bufferView"]]
                    assert accessor["componentType"] == 5126 and accessor["type"] == "VEC3"
                    start = view.get("byteOffset", 0) + accessor.get("byteOffset", 0)
                    stride = view.get("byteStride", 12)
                    positions[stage, mesh["name"]] = [
                        struct.unpack_from("<3f", data, start + i * stride) for i in range(accessor["count"])
                    ]
            assert buffers[stage, "matte"] == buffers[stage, "finished"], stage
        moved = {}
        for stage, name in positions:
            if stage == "rest":
                pairs = zip(positions["rest", name], positions["step", name], strict=True)
                moved[name] = sum(a != b for a, b in pairs)
                assert moved[name] > 0
        artifacts = Path(__file__).parent
        native_project = json.loads((artifacts / "Harbor finish.weftforge").read_bytes())
        native_revision = native_project["project"]["revision"]
        assert native_revision == source["project"]["revision"] + 4
        expected_native = json.loads(finished.read_bytes())
        expected_native["project"]["revision"] = native_revision
        assert expected_native == native_project
        before, before_data = read_scene(artifacts / "Before finish.gltf")
        after, after_data = read_scene(artifacts / "After finish.gltf")
        assert before_data == after_data == buffers["step", "finished"]
        assert before["asset"]["extras"]["weftforgeSourceRevision"] == str(source["project"]["revision"])
        assert after["asset"]["extras"]["weftforgeSourceRevision"] == str(native_revision)
        assert before["meshes"] == after["meshes"]
        for scene, pair in [(before, (0, 1)), (after, (0.625, 0.25))]:
            assert len(scene["materials"]) == 2
            for material in scene["materials"]:
                factors = material["pbrMetallicRoughness"]
                assert (factors["metallicFactor"], factors["roughnessFactor"]) == (
                    pair if material["extras"]["weftforgeMaterialId"] == "68" else (0, 1)
                )
        assert FIXTURE.read_bytes() == original
        print(json.dumps({
            "fixture_sha256": sha256(original).hexdigest(),
            "native_before_after_buffer_equals_cli_step": True,
            "native_document_revision": native_revision,
            "native_before_after_buffer_bytes": len(after_data),
            "material_ids": [1, 68],
            "saved_version": saved["version"],
            "moved_vertices_after_step": moved,
            "glb_and_gltf_scene_and_buffer_equal": True,
            "pre_and_post_finish_buffer_sha256": {
                stage: sha256(buffers[stage, "matte"]).hexdigest() for stage in ("rest", "step")
            }
        }, indent=2))


if __name__ == "__main__":
    check(Path(sys.argv[1]).resolve())
