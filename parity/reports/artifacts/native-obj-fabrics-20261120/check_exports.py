"""Replay the native OBJ material case. Pass --cli to repeat both release adapters."""
import argparse
import hashlib
import json
from math import isclose
from pathlib import Path
import subprocess
import tempfile

from check_fabrics import check, read_surface


CASE = Path(__file__).resolve().parent
TWO = CASE / "Harbor.obj"
ONE = CASE / "All terracotta.obj"
RESTORED = CASE / "Restored Harbor.weftforge"


def read(path):
    return json.loads(path.read_bytes())


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def library(obj):
    names = [line[7:] for line in obj.read_text().splitlines() if line.startswith("mtllib ")]
    assert len(names) == 1
    path = obj.with_name(names[0])
    assert path.name == f"weftforge-fabrics-{digest(path)}.mtl"
    return path


def geometry_lines(obj):
    return [line for line in obj.read_text().splitlines()
            if not line.startswith(("mtllib ", "usemtl "))]


def check_modelio(receipt, surface):
    rows = read(receipt)
    assignments = {}
    for row in rows:
        name = row["material"]
        assert row["vertices"] == surface["positions"] and row["indices"] == 586 * 3
        rgb = surface["materials"][name]["diffuse_rgb"]
        assert len(row["diffuse_rgb"]) == 3
        assert all(isclose(x, y, abs_tol=1e-6) for x, y in zip(row["diffuse_rgb"], rgb))
        assert row["submesh"].endswith("_" + name)
        group = row["submesh"].removesuffix("_" + name)
        assert group not in assignments
        assignments[group] = name
    assert assignments == {group: name for name, data in surface["materials"].items()
                           for group in data["groups"]}


def replay_adapters(exe, directory):
    fixture = directory / "fixture.weftforge"
    baseline = CASE.parent / "native-hem-regeneration-20261120/full-fit/original/Harbor.weftforge"
    subprocess.run([exe, "execute", baseline, fixture, CASE / "two-fabrics.json"],
                   check=True, capture_output=True, timeout=30)
    assert fixture.read_bytes() == (CASE / "Harbor two fabrics.weftforge").read_bytes()
    native = {path.name: digest(path) for path in (TWO, library(TWO))}
    for adapter in ("cli", "mcp"):
        work = directory / adapter
        work.mkdir()
        output = work / "Harbor.obj"
        if adapter == "cli":
            subprocess.run([exe, "export-obj", RESTORED, output], check=True,
                           capture_output=True, timeout=30)
        else:
            message = {"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {
                "_meta": {"io.modelcontextprotocol/protocolVersion": "2026-07-28",
                          "io.modelcontextprotocol/clientCapabilities": {}},
                "name": "project_export_obj",
                "arguments": {"input": str(RESTORED), "output": str(output)}}}
            response = subprocess.run([exe, "mcp"], input=json.dumps(message) + "\n",
                                      text=True, capture_output=True, check=True, timeout=30)
            result = json.loads(response.stdout)["result"]
            assert not result["isError"]
            content = result["structuredContent"]
            assert content["source_revision"] == 26
            assert Path(content["material_library"]).resolve() == library(output).resolve()
        assert {p.name: digest(p) for p in work.iterdir()} == native


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cli", type=Path, help="Path to the WeftForge CLI executable")
    args = parser.parse_args()
    fixture = CASE / "Harbor two fabrics.weftforge"
    original = read(fixture)
    restored = read(RESTORED)
    assert original["project"]["revision"] == 24 and restored["project"]["revision"] == 26
    original["project"]["revision"] = 26
    assert original == restored
    two = check(RESTORED, TWO)
    single = read_surface(ONE)
    assert geometry_lines(ONE) == geometry_lines(TWO)
    assert single["triangles"] == two["triangles"] == 1172
    assert len(single["materials"]) == 1
    linen = single["materials"]["fabric_68_Terracotta_linen"]
    assert linen["groups"] == ["piece_2", "piece_31"] and linen["diffuse_rgb"] == [0.78, 0.25, 0.12]
    assert isclose(linen["area_m2"], sum(v["area_m2"] for v in two["materials"].values()))
    assert library(ONE) != library(TWO)
    for obj, surface in ((ONE, single), (TWO, two)):
        definitions = [line[7:] for line in library(obj).read_text().splitlines()
                       if line.startswith("newmtl ")]
        assert sorted(definitions) == sorted(surface["materials"])
    for name, surface in [("single", single), ("two", two)]:
        check_modelio(CASE / f"MODELIO-{name}.json", surface)
    native = read(CASE / "NATIVE.json")
    assert digest(TWO) == native["two_fabric_obj_sha256"] and digest(ONE) == native["single_fabric_obj_sha256"]
    assert digest(fixture) == native["input_sha256"]
    adapters = read(CASE / "ADAPTERS.json")
    assert adapters["source_revision"] == 26
    assert adapters["byte_identical_files"] == {p.name: digest(p) for p in (TWO, library(TWO))}
    assert adapters["mcp_companion_filename"] == library(TWO).name
    observed = read(CASE.parents[3] / "spec/observations/REF-FILE-003-OBJ-FABRICS-001.json")
    assert digest(TWO) == observed["original_input_sha256"]
    for name, row in observed["measurements"]["by_assigned_fabric"].items():
        assert two["materials"][name] == row["input"]
    if args.cli:
        before = digest(RESTORED)
        with tempfile.TemporaryDirectory() as temp:
            replay_adapters(args.cli.resolve(), Path(temp))
        assert digest(RESTORED) == before
    print(json.dumps({"two_fabric": two, "single_fabric": single,
                      "mesh_records_identical": True, "restored_except_revision": True,
                      "checked_modelio_submeshes_per_export": 2}, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
