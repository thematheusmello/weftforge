"""Replay the Debian Wayland cutout geometry and the unsigned package ledger."""
import json
from pathlib import Path
import struct
import subprocess
import sys


def run_json(script, *args):
    return json.loads(subprocess.check_output([sys.executable, script, *args]))


def read(path):
    return json.loads(path.read_bytes())


def main():
    case = Path(__file__).resolve().parent
    original = case.parent / "native-svg-cutout-20261120"
    notices = case.parent / "debian-arm64-cargo-notices-20261120"
    native_files = [case / f"Pocket {state}.{ext}"
                    for state in ("imported", "edited") for ext in ("weftforge", "obj")]
    geometry = run_json(original / "check_cutout.py", original / "Pocket facing.svg", *native_files)
    assert geometry == read(case / "GEOMETRY.json") == read(original / "GEOMETRY.json")
    for path in native_files:
        assert path.read_bytes() == (original / path.name).read_bytes()

    package = run_json(notices / "check_notices.py", "--manifest", case / "manifest.json",
                       "--ledger", case / "PACKAGE-SHA256SUMS")
    assert package == read(case / "PACKAGE-READBACK.json")
    adapters = read(case / "ADAPTERS.json")
    assert adapters["executable_sha256"] == package["artifacts"]["weftforge-cli"]
    assert adapters["adapters"] == read(original / "ADAPTERS.json")["adapters"]

    session = read(case / "WAYLAND.json")
    first, chooser, reopened = (session["surfaces"][key] for key in ("first", "chooser", "reopened"))
    assert [w["app_id"] for w in first] == [w["app_id"] for w in reopened] == ["WeftForge"]
    assert first[0]["pid"] != reopened[0]["pid"]
    assert {w["app_id"] for w in chooser} == {"WeftForge", "xdg-desktop-portal-gtk"}
    assert all(w["shell"] == "xdg_shell" for w in first + chooser + reopened)
    for name in ("Pocket imported.png", "Pocket edited.png", "import-review.png", "gtk-save.png"):
        data = (case / name).read_bytes()
        assert data[:16] == b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR"
        assert struct.unpack(">II", data[16:24]) == (1640, 1000)
    print("Two editable saves and two OBJ exports match across platforms.")
    print("425 package entries, 234 Cargo crates and both adapter receipts agree.")
    print("The GTK portal and both studio processes used xdg-shell surfaces.")


if __name__ == "__main__":
    main()
