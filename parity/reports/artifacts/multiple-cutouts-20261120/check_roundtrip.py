"""Replay both file adapters and measure the two-slot physical mesh independently."""

import argparse
from collections import Counter, defaultdict
import hashlib
import json
import math
from pathlib import Path
import subprocess
import tempfile

CASE = Path(__file__).resolve().parent
SOURCE = CASE / "Pocket two slots.svg"
OVERLAP = CASE / "Overlapping slots.svg"


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def geometry(path):
    points, uv, triangles = [], [], []
    for row in path.read_text().splitlines():
        cols = row.split()
        if not cols:
            continue
        if cols[0] == "v":
            points.append([float(v) for v in cols[1:]])
        elif cols[0] == "vt":
            uv.append([float(v) for v in cols[1:]])
        elif cols[0] == "f":
            indices = [
                tuple(int(i) - 1 for i in group.split("/")) for group in cols[1:]
            ]
            assert len(indices) == 3 and all(a == b == c for a, b, c in indices)
            triangles.append([i[0] for i in indices])
    assert len(points) == len(uv) and all(
        p[:2] == t and p[2] == 0 for p, t in zip(points, uv)
    )
    edges = Counter()
    area = 0.0
    for tri in triangles:
        z = [points[i] for i in tri]
        a, b, c = z
        signed = ((b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])) / 2
        assert signed > 0
        area += signed
        for i in range(3):
            edges[tuple(sorted((tri[i], tri[(i + 1) % 3])))] += 1
    boundary = defaultdict(set)
    for (a, b), count in edges.items():
        assert count in (1, 2)
        if count == 1:
            boundary[a].add(b)
            boundary[b].add(a)
    assert all(len(v) == 2 for v in boundary.values())
    lengths = []
    remaining = set(boundary)
    while remaining:
        stack = [next(iter(remaining))]
        length = 0.0
        while stack:
            v = stack.pop()
            if v in remaining:
                remaining.remove(v)
                for u in boundary[v]:
                    length += math.dist(points[v], points[u]) / 2
                    stack.append(u)
        lengths.append(length)
    lengths.sort()
    euler = len(points) - len(edges) + len(triangles)
    assert len(lengths) == 3 and euler == -1
    assert all(
        math.isclose(a, b, abs_tol=1e-12) for a, b in zip(lengths, [0.14, 0.40, 1.20])
    )
    assert math.isclose(area, 0.0756, abs_tol=1e-12)
    return {
        "vertices": len(points),
        "faces": len(triangles),
        "boundary_perimeters_m": lengths,
        "euler_characteristic": euler,
        "physical_area_m2": area,
    }


def adapters(exe, directory):
    def cli(*args):
        return subprocess.run([exe, *args], capture_output=True, text=True, timeout=45)

    def mcp(tool, **args):
        request = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "_meta": {
                    "io.modelcontextprotocol/protocolVersion": "2026-07-28",
                    "io.modelcontextprotocol/clientCapabilities": {},
                },
                "name": tool,
                "arguments": {k: str(v) for k, v in args.items()},
            },
        }
        reply = subprocess.run(
            [exe, "mcp"],
            input=json.dumps(request) + "\n",
            text=True,
            capture_output=True,
            check=True,
            timeout=45,
        )
        return json.loads(reply.stdout)["result"]

    empty = directory / "empty.weftforge"
    cli("new", empty).check_returncode()
    for route in ["cli", "mcp"]:
        folder = directory / route
        folder.mkdir()
        saved, mesh, drawing = (
            folder / name
            for name in ["Pocket.weftforge", "Pocket.obj", "Roundtrip.svg"]
        )
        if route == "cli":
            result = cli("import-svg", empty, saved, SOURCE)
            result.check_returncode()
            info = json.loads(result.stdout)
            cli("export-obj", saved, mesh).check_returncode()
            cli("export-svg", saved, drawing).check_returncode()
        else:
            result = mcp("project_import_svg", input=empty, output=saved, svg=SOURCE)
            assert not result["isError"]
            info = result["structuredContent"]
            assert not mcp("project_export_obj", input=saved, output=mesh)["isError"]
            assert not mcp("project_export_svg", input=saved, output=drawing)["isError"]
        assert info["pieces_imported"] == 1 and info["holes_imported"] == 2
        for p in [saved, mesh, drawing, *folder.glob("*.mtl")]:
            assert digest(p) == digest(CASE / p.name), p.name
        reimport = folder / "Reopened.weftforge"
        cli("import-svg", empty, reimport, drawing).check_returncode()
        assert reimport.read_bytes() == saved.read_bytes()
        recovery = saved.with_suffix(".weftforge.recovery")
        recovery.write_bytes(b"original recovery")
        before = [digest(p) for p in [saved, recovery, empty, OVERLAP]]
        if route == "cli":
            assert cli("import-svg", empty, saved, OVERLAP).returncode != 0
        else:
            assert mcp("project_import_svg", input=empty, output=saved, svg=OVERLAP)[
                "isError"
            ]
        assert before == [digest(p) for p in [saved, recovery, empty, OVERLAP]]


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--cli", type=Path)
    args = parser.parse_args()
    metrics = geometry(CASE / "Pocket.obj")
    if args.cli:
        with tempfile.TemporaryDirectory() as directory:
            adapters(args.cli.resolve(), Path(directory))
    print(json.dumps(metrics, sort_keys=True, indent=2))
