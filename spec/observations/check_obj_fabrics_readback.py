"""Check a separately retained OBJ/MTL return against the reviewed two-fabric observation."""
import argparse
import hashlib
import json
from math import isclose
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "parity/reports/artifacts/native-obj-fabrics-20261120"))
from check_fabrics import read_surface


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("returned_obj", type=Path)
    args = parser.parse_args()
    observed = json.loads((HERE / "REF-FILE-003-OBJ-FABRICS-001.json").read_bytes())
    obj = args.returned_obj
    libraries = [line[7:].strip() for line in obj.read_text().splitlines() if line.startswith("mtllib ")]
    assert len(libraries) == 1
    for key, path in (("obj", obj), ("mtl", obj.with_name(libraries[0]))):
        assert hashlib.sha256(path.read_bytes()).hexdigest() == observed["reference_output_sha256"][key]
    returned = read_surface(obj)
    measurements = observed["measurements"]
    assert [returned["positions"], returned["triangles"]] == measurements["returned"]
    by_color = {tuple(data["diffuse_rgb"]): data for data in returned["materials"].values()}
    assert len(by_color) == len(measurements["by_assigned_fabric"]) == 2
    for values in measurements["by_assigned_fabric"].values():
        source = values["input"]
        expected = values["returned"]
        found = by_color[tuple(source["diffuse_rgb"])]
        assert found["diffuse_rgb"] == expected["diffuse_rgb"]
        assert found["triangles"] == expected["triangles"]
        assert found["groups"] == expected["groups"] == []
        for quantity, delta in (("area_m2", "area_delta_m2"), ("boundary_m", "perimeter_delta_m")):
            assert isclose(found[quantity], expected[quantity], rel_tol=0, abs_tol=1e-12)
            assert isclose(found[quantity] - source[quantity], values[delta], rel_tol=0, abs_tol=1e-12)
    print("The retained return matches both recorded colors, areas, perimeters and triangle counts.")


if __name__ == "__main__":
    main()
