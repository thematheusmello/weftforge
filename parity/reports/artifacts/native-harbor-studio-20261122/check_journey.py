"""Replay the construction and fitted surface measurements from the repository root."""
import json
from pathlib import Path
import struct
import sys

HERE = Path(__file__).resolve().parent
ROOT = Path.cwd()
sys.path.insert(0, str(ROOT / "scripts"))
from check_drape import measure

def project(path):
    return json.loads(path.read_bytes())["project"]

def check_height(original, path, height):
    edited = project(path)
    front = original["pieces"][0]
    endpoints = {
        curve["start"] for curve in original["curves"] if curve["id"] in front["boundary"]
    }
    ys = [point["position"][1] for point in original["points"] if point["id"] in endpoints]
    assert len(ys) == 14
    for old, new in zip(original["points"], edited["points"], strict=True):
        assert old["id"] == new["id"]
        x, y = old["position"]
        wanted = min(ys) + (y - min(ys)) * height / (max(ys) - min(ys))
        if old["id"] in endpoints:
            assert abs(x - new["position"][0]) < 1e-12 and abs(new["position"][1] - wanted) < 1e-12
        else:
            assert new == old
    assert edited.keys() == original.keys()
    assert all(edited[key] == original[key] for key in edited if key not in {"revision", "points"})
    return edited["revision"]

def main():
    authored = HERE / "Harbor authored.weftforge"
    expected = ROOT / "parity/reports/artifacts/native-harbor-workflow-20261118/arranged-and-sewn.weftforge"
    original = project(authored)
    assert original == project(expected)
    assert len(original["points"]) == 28 and len(original["seams"]) == 8
    edited = HERE / "Harbor 670.weftforge"
    revisions = [original["revision"], check_height(original, HERE / "Harbor 680.weftforge", .68),
                 check_height(original, edited, .67)]
    assert revisions == [16, 17, 18]
    captures = {}
    for name in ("empty-default", "empty-minimum", "empty-minimum-retina", "drafting",
                 "sewing-refusal", "sewing-direction", "keyboard-focus", "fitted-default",
                 "fitted-minimum", "reopened-default", "reopened-minimum", "reopened-fit"):
        data = (HERE / (name + ".png")).read_bytes()
        assert data[:8] == b"\x89PNG\r\n\x1a\n" and data[12:16] == b"IHDR"
        size = struct.unpack(">II", data[16:24])
        expected_size = (1640, 1032)
        if "minimum" in name:
            expected_size = (2000, 1364) if "retina" in name else (1000, 682)
        assert size == expected_size, (name, size)
        captures[name] = size
    result = {"construction_revisions": revisions, "front_heights_m": [.64, .68, .67],
              "window_pixels_including_title_bar": captures,
              "original_export": measure(authored, HERE / "Harbor fitted.obj"),
              "reopened_export": measure(edited, HERE / "Harbor reopened fitted.obj")}
    print(json.dumps(result, indent=2))

if __name__ == "__main__":
    main()
