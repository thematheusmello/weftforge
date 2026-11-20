"""Recheck the captured Debian Save As output against its original construction."""

import hashlib
import json
from pathlib import Path


case = Path(__file__).resolve().parent
original = case.parent / "pattern-png-grainline-20261119" / "rotated-basin.weftforge"
saved = case / "linux-native-save.weftforge"
receipt = json.loads((case / "NATIVE-SAVE.json").read_bytes())
readback = receipt["independent_readback"]

data = saved.read_bytes()
assert data == original.read_bytes()
digest = hashlib.sha256(data).hexdigest()
for field in ("original_protected_fixture_sha256", "loaded_copy_sha256", "saved_file_sha256"):
    assert digest == readback[field], field

envelope = json.loads(data)
project = envelope["project"]
assert envelope["format"] == "weftforge"
assert envelope["version"] == readback["envelope_version"] == 6
assert project["revision"] == readback["revision"] == 3
assert [p["name"] for p in project["pieces"]] == readback["patterns"] == ["Juniper α", "Basin β"]
assert len(project["points"]) == readback["points"] == 14
assert len(project["curves"]) == readback["curves"] == 13
assert len(project["seams"]) == readback["seams"] == 0
print(f"Verified {len(data)} identical bytes, {len(project['pieces'])} patterns, revision 3, SHA-256 {digest}")
