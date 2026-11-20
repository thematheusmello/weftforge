"""Check a selected OBJ against the full surface and exercise both file adapters."""

import argparse
import hashlib
import json
import math
import sys
from pathlib import Path
import subprocess
import tempfile


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def decode(path):
    pools = [[] for _ in range(3)]
    used = [set() for _ in pools]
    faces, materials = {}, {}
    group = library = material = None
    for line in path.read_text().splitlines():
        parts = line.split()
        if not parts:
            continue
        tag, data = parts[0], parts[1:]
        if tag in ("v", "vt", "vn"):
            pools[("v", "vt", "vn").index(tag)].append(tuple(map(float, data)))
        elif tag == "g":
            (group,) = data
            assert group not in faces
            faces[group] = []
        elif tag == "mtllib":
            name = " ".join(data)
            assert library is None and Path(name).name == name
            library = path.with_name(name)
        elif tag == "usemtl":
            (material,) = data
            materials[group] = material
        elif tag == "f":
            assert group and material and len(data) == 3
            face = []
            for corner in data:
                indices = tuple(int(i) - 1 for i in corner.split("/"))
                assert len(indices) == 3
                for pool, seen, i in zip(pools, used, indices):
                    assert 0 <= i < len(pool)
                    seen.add(i)
                face.append(tuple(pool[i] for pool, i in zip(pools, indices)))
            faces[group].append(tuple(face))
    assert all(seen == set(range(len(pool))) for seen, pool in zip(used, pools))
    assert library and library.name == f"weftforge-fabrics-{sha(library)}.mtl"
    definitions = {
        line[7:]
        for line in library.read_text().splitlines()
        if line.startswith("newmtl ")
    }
    assert definitions == set(materials.values())
    return faces, materials, library


def compare(selected, whole, groups):
    subset, materials, _ = decode(selected)
    full, full_materials, _ = decode(whole)
    assert list(subset) == groups
    for group in groups:
        assert subset[group] == full[group]
        assert materials[group] == full_materials[group]
    return {group: len(faces) for group, faces in subset.items()}


def adapters(exe, input_path, whole, single):
    def cli(*args):
        return subprocess.run([exe, *map(str, args)], capture_output=True, timeout=30)

    def mcp(params):
        req = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "project_export_obj",
                "arguments": params,
                "_meta": {
                    "io.modelcontextprotocol/protocolVersion": "2026-07-28",
                    "io.modelcontextprotocol/clientCapabilities": {},
                },
            },
        }
        run = subprocess.run(
            [exe, "mcp"],
            input=json.dumps(req) + "\n",
            text=True,
            capture_output=True,
            check=True,
            timeout=30,
        )
        return json.loads(run.stdout)["result"]

    with tempfile.TemporaryDirectory() as name:
        folder = Path(name)
        for ids, expected in [([31], single), ([2, 31], whole), ([31, 2], whole)]:
            a, b = folder / "cli.obj", folder / "mcp.obj"
            cli("export-obj", input_path, a, *ids).check_returncode()
            params = {"input": str(input_path), "output": str(b), "pieces": ids}
            reply = mcp(params)
            assert not reply["isError"]
            assert reply["structuredContent"]["pieces"] == ids
            assert a.read_bytes() == b.read_bytes() == expected.read_bytes()
            assert Path(reply["structuredContent"]["material_library"]) == decode(b)[2]
        protected = folder / "existing.obj"
        protected.write_bytes(b"original destination")
        for bad in ([], [0], [31, 31], [2, 999999], None):
            assert mcp(
                {"input": str(input_path), "output": str(protected), "pieces": bad}
            )["isError"]
            if bad:
                assert cli("export-obj", input_path, protected, *bad).returncode
            assert protected.read_bytes() == b"original destination"
        for dest in (input_path, input_path.with_suffix(".weftforge.recovery")):
            assert mcp({"input": str(input_path), "output": str(dest), "pieces": [31]})[
                "isError"
            ]
            assert cli("export-obj", input_path, dest, 31).returncode


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("whole", type=Path)
    parser.add_argument("back", type=Path)
    parser.add_argument("--cli", type=Path)
    parser.add_argument("--project", type=Path)
    parser.add_argument(
        "--modelio",
        type=Path,
        help="Verify a saved Model I/O readback for the selected OBJ.",
    )
    parser.add_argument(
        "--reference-return",
        type=Path,
        help="Verify the separately retained licensed reference OBJ and MTL.",
    )
    options = parser.parse_args()
    result = compare(options.back, options.whole, ["piece_31"])
    if options.cli:
        assert options.project
        before = sha(options.project)
        adapters(
            options.cli.resolve(),
            options.project.resolve(),
            options.whole,
            options.back,
        )
        assert sha(options.project) == before
    if options.modelio:
        rows = json.loads(options.modelio.read_bytes())
        assert len(rows) == 1
        row = rows[0]
        assert row["indices"] == result["piece_31"] * 3 and row["vertices"] == 327
        assert row["material"] == "fabric_68_Terracotta_linen"
        assert all(
            abs(a - b) < 1e-6
            for a, b in zip(row["diffuse_rgb"], [0.78, 0.25, 0.12], strict=True)
        )
    if options.reference_return:
        here = Path(__file__).resolve().parent
        sys.path.insert(0, str(here.parent / "native-obj-fabrics-20261120"))
        from check_fabrics import read_surface

        obs = json.loads(
            (
                here.parents[3] / "spec/observations/REF-FILE-003-OBJ-SELECTED-001.json"
            ).read_bytes()
        )
        reference_obj = options.reference_return
        mtls = [
            line[7:].strip()
            for line in reference_obj.read_text().splitlines()
            if line.startswith("mtllib ")
        ]
        assert len(mtls) == 1 and Path(mtls[0]).name == mtls[0]
        assert {
            "obj": sha(reference_obj),
            "mtl": sha(reference_obj.with_name(mtls[0])),
        } == obs["returned_hashes"]
        measured = read_surface(reference_obj)
        assert measured == obs["returned_surface"] and len(measured["materials"]) == 1
        selected = next(iter(measured["materials"].values()))
        native = next(iter(read_surface(options.back)["materials"].values()))
        assert native["diffuse_rgb"] == selected["diffuse_rgb"]
        assert math.isclose(
            selected["area_m2"] - native["area_m2"], obs["delta_area_m2"], abs_tol=1e-12
        )
        assert math.isclose(
            selected["boundary_m"] - native["boundary_m"],
            obs["delta_perimeter_m"],
            abs_tol=1e-12,
        )
    print(json.dumps(result, sort_keys=True))
