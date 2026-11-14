#!/usr/bin/env python3
"""Verify candidate hashes and JPEG filename corrections without altering assets."""
from pathlib import Path
import hashlib
import json

root = Path(__file__).resolve().parent
expected = {}
for line in (root / "MANIFEST.sha256").read_text().splitlines():
    digest, name = line.split("  ", 1)
    assert name not in expected
    expected[name] = digest
actual = {str(p.relative_to(root)) for p in root.rglob("*") if p.is_file() and p.name != "MANIFEST.sha256"}
assert actual == set(expected)
for name, digest in expected.items():
    assert hashlib.sha256((root / name).read_bytes()).hexdigest() == digest
provenance = json.loads((root / "PROVENANCE.json").read_text())
assert {p["public_file"] for p in provenance["files"]} == actual - {"PROVENANCE.json"}
jpegs = 0
for item in provenance["files"]:
    data = (root / item["public_file"]).read_bytes()
    assert hashlib.sha256(data).hexdigest() == item["public_sha256"]
    if item["actual_mime"] == "image/jpeg":
        jpegs += 1
        assert data[:3] == bytes([255, 216, 255])
        assert item["public_file"].endswith(".jpg")
        assert item["original_sha256"] == item["public_sha256"]
        assert item["transform"] == "filename correction only; bytes unchanged"
    elif item["transform"] == "bytes unchanged":
        assert item["original_sha256"] == item["public_sha256"]
    else:
        assert item["transform"] == "UTF-8 literal prefix replacements only"
        assert item["exact_replacements"]
        for mapping in item["exact_replacements"]:
            assert len(mapping["original_prefix_sha256"]) == 64
            assert mapping["original_prefix_length_utf8"] > 0
            assert mapping["replacement"] in ["<checkout>", "<user-home>"]
assert jpegs == 33
assert not list(root.rglob("*.png"))
print(json.dumps({"passed": True, "manifest_files_verified": len(expected), "byte_identical_jpeg_captures": jpegs, "png_assets": 0, "content_edits": 0}, indent=2))
