"""Check the published Debian package records and independently decode its native JPEGs."""

import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
from tempfile import TemporaryDirectory


case = Path(__file__).resolve().parent
jpeg_controls = case.parent / "pattern-jpeg-20261119"
original = case.parent / "pattern-png-grainline-20261119" / "rotated-basin.weftforge"
saved = case / "current-source.weftforge"
receipt = json.loads((case / "NATIVE-SAVE-JPEG.json").read_bytes())
assert saved.read_bytes() == original.read_bytes()
assert hashlib.sha256(saved.read_bytes()).hexdigest() == receipt["written_project"]["sha256"]
assert [event["response_code"] for event in receipt["portal"]["events"]] == [0, 0, 2, 0]
assert [event["selected_basename"] for event in receipt["portal"]["events"]] == [
    "current-source.weftforge", "fit.jpg", None, "two-metre.jpeg"
]

manifest = case / "manifest.json"
package = json.loads(manifest.read_bytes())
readback = json.loads((case / "PACKAGE-READBACK.json").read_bytes())
rows = [line.split("  ", 1) for line in (case / "PACKAGE-SHA256SUMS").read_text().splitlines()]
inventory = {name.removeprefix("./"): digest for digest, name in rows}
assert len(rows) == len(inventory) == receipt["actual_package_files"]
assert hashlib.sha256(manifest.read_bytes()).hexdigest() == inventory["licenses/manifest.json"]
assert package["target"] == readback["target"] == "aarch64-unknown-linux-gnu"
records = [*package["artifacts"], *package["project_notices"]]
unresolved = {}
for crate in package["crates"]:
    records.extend(crate["files"])
    if crate["unresolved"]:
        unresolved[crate["name"]] = crate["unresolved"]
assert unresolved == readback["unresolved"] and len(unresolved) == 7
assert len(package["crates"]) == readback["emitted_libraries"] == 234
for entry in records:
    assert inventory[entry["path"]] == entry["sha256"], entry["path"]
recorded = {entry["path"] for entry in records}
assert len(recorded) == readback["checked"] == 415
assert set(inventory) == recorded | {"README.md", "licenses/manifest.json"}
assert {entry["path"]: entry["sha256"] for entry in package["artifacts"]} == receipt["executables"]

checker = jpeg_controls / "verify_pattern_jpeg.py"
geometry = json.loads((case / "JPEG-GEOMETRY.json").read_bytes())
adapters = json.loads((case / "CLI-MCP.json").read_bytes())
assert hashlib.sha256(checker.read_bytes()).hexdigest() == geometry["checker_sha256"]
assert adapters["cli_sha256"] == receipt["executables"]["weftforge-cli"]
with TemporaryDirectory() as name:
    folder = Path(name)
    shutil.copyfile(checker, folder / checker.name)
    for options in ("fit.json", "two-metre.json"):
        shutil.copyfile(jpeg_controls / options, folder / options)
    for native, adapter in zip(geometry["outputs"], adapters["outputs"], strict=True):
        filename = native["file"]
        exported = case / filename
        assert exported.read_bytes() == (jpeg_controls / filename).read_bytes()
        assert adapter["file"] == filename and adapter["mcp_byte_equal"]
        assert hashlib.sha256(exported.read_bytes()).hexdigest() == adapter["sha256"] == native["sha256"]
        shutil.copyfile(exported, folder / filename)
    output = subprocess.check_output([sys.executable, folder / checker.name, "--fixture", saved])
    assert json.loads(output)["outputs"] == geometry["outputs"]

print("Verified the editable project, two native JPEGs and all 415 unique package manifest records.")
