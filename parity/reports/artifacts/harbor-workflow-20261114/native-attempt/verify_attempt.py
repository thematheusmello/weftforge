#!/usr/bin/env python3
"""Read-only identity, capped-state, JPEG and inventory checks; no quality-pass claim."""
from pathlib import Path
import hashlib
import json
import math
root = Path(__file__).resolve().parent
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
expected = {}
for line in (root / "MANIFEST.sha256").read_text().splitlines():
    digest, name = line.split("  ", 1)
    assert name not in expected
    expected[name] = digest
actual = {str(p.relative_to(root)) for p in root.rglob("*") if p.is_file() and p.name != "MANIFEST.sha256"}
assert actual == set(expected)
for name, digest in expected.items():
    assert sha(root / name) == digest
prov = json.loads((root / "PROVENANCE.json").read_text())
assert {p["public_file"] for p in prov["files"]} == actual - {"PROVENANCE.json"}
jpegs = 0
for item in prov["files"]:
    data = (root / item["public_file"]).read_bytes()
    assert hashlib.sha256(data).hexdigest() == item["public_sha256"]
    if item["actual_mime"] == "image/jpeg":
        jpegs += 1
        assert data[:3] == bytes([255, 216, 255]) and item["public_file"].endswith(".jpg")
        assert item["original_sha256"] == item["public_sha256"]
        assert item["transform"] == "bytes unchanged"
    elif item["transform"] == "bytes unchanged":
        assert item["original_sha256"] == item["public_sha256"]
    else:
        assert item["transform"] == "UTF-8 literal prefix replacements only"
        assert item["exact_replacements"]
assert jpegs == 8 and not list(root.rglob("*.png"))
a = json.loads((root / "base/case-receipt.json").read_text())
b = json.loads((root / "height/case-receipt.json").read_text())
assert a["actual_accepted_checkpoints"] == [0, 60] and a["missing_checkpoints"] == [180]
assert a["AX_observation_overrun_s"] > 0 and a["wall_cap_s"] == 420
assert b["status"] == "not_run" and b["simulation_batches_started"] == 0
for name in ["stage-000.weftforge", "stage-060.weftforge", "cap-retained.weftforge"]:
    assert sha(root / "base" / name) == "fc494974955d440aacbfde49c332a50218bd3a9354d06f4fbbf0596e5da8eda7"
assert (root / "base/cap-retained.obj").read_bytes() == (root / "base/stage-060.obj").read_bytes()
assert not list(root.rglob("stage-180.obj"))
for file in ["stage-000.obj", "stage-060.obj", "cap-retained.obj"]:
    for line in (root / "base" / file).read_text().splitlines():
        if line.startswith(("v ", "vt ", "vn ")):
            assert all(math.isfinite(float(s)) for s in line.split()[1:])
print(json.dumps({"passed": True, "manifest_files_verified": len(expected), "raw_jpegs_verified": jpegs, "A_accepted_frames": 60, "A_frame180": "missing_capped", "B": "not_run", "quality": "full_protocol_incomplete_no_final_quality_pass"}, indent=2))
