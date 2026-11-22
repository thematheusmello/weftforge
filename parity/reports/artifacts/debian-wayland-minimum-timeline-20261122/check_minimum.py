"""Read the native captures, saved pattern and package ledger independently."""
import hashlib
import json
from pathlib import Path
import re
import struct
import subprocess
import sys

HERE = Path(__file__).resolve().parent


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def read(path):
    return json.loads(path.read_bytes())


def main():
    run = read(HERE / "RUN.json")
    package = read(HERE / "PACKAGE-READBACK.json")
    manifest = read(HERE / "manifest.json")
    ledger_file = HERE / "PACKAGE-SHA256SUMS"
    ledger = dict(line.split("  ./", 1)[::-1] for line in ledger_file.read_text().splitlines())
    assert sha(ledger_file) == package["ledger_sha256"]
    assert sha(HERE / "manifest.json") == ledger["licenses/manifest.json"] == package["manifest_sha256"]
    assert sha(HERE / "BUILD-SOURCE-SHA256SUMS") == run["source_inputs_sha256"]
    assert len(ledger) == package["verified_files"] == 443
    assert len(manifest["crates"]) == package["emitted_cargo_crates"] == 243
    assert ledger["weftforge"] == run["studio_sha256"] == package["executables"]["weftforge"]
    records = [*manifest["artifacts"], *manifest["project_notices"]]
    for crate in manifest["crates"]:
        assert not crate["unresolved"]
        records.extend(crate["files"])
    assert all(record["sha256"] == ledger[record["path"]] for record in records)
    original = HERE.parent / "native-timeline-20261122/Swatch.weftforge"
    assert sha(original) == run["fixture_sha256"]
    heights = []
    for path, expected in [(original, (3, 0.36)), (HERE / "resized.weftforge", (4, 0.37))]:
        document = read(path)
        project = document["project"]
        assert document["version"] == 8 and project["revision"] == expected[0]
        ys = [point["position"][1] for point in project["points"]]
        height = max(ys) - min(ys)
        assert abs(height - expected[1]) < 1e-6
        heights.append(height)
    screens = run["retained_screens"]
    assert len({screen["sha256"] for screen in screens.values()}) == len(screens) == 4
    for screen in screens.values():
        image = HERE / screen["filename"]
        assert sha(image) == screen["sha256"]
        assert screen["focused"] and screen["app_id"] == "WeftForge"
        assert screen["rect"] == {"x": 0, "y": 0, "width": 1000, "height": 650}
        assert struct.unpack(">II", image.read_bytes()[16:24]) == (1000, 650)
    if "--ocr" in sys.argv:
        for frame, patterns in [
            ("recording-four", ["Recording . 4 frames"]),
            ("before-keyboard", ["4 recorded", "Time 0.050 s"]),
            ("after-keyboard", ["Viewing frame 3 at 0.033333 s . latest solver 0.050000 s", "Latest solver state"]),
            ("height-edit", ["Rest arrangement", "Saved editable project atomically"]),
        ]:
            image = HERE / screens[frame]["filename"]
            text = subprocess.check_output(["tesseract", str(image), "stdout", "--psm", "11"], stderr=subprocess.DEVNULL, text=True)
            for pattern in patterns:
                assert re.search(pattern, " ".join(text.split()), re.I), (frame, pattern, text)
    print(f"Four 1000 × 650 captures; heights {heights[0]:.6f} and {heights[1]:.6f} m; "
          f"{len(ledger)} package files and {len(manifest['crates'])} emitted crates.")


if __name__ == "__main__":
    main()
