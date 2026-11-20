"""Replay the retained editable saves, OBJ geometry, Wayland captures and package ledger."""

import json
from pathlib import Path
import struct

from check_slots import check_case, digest


CASE = Path(__file__).resolve().parent


def read(name):
    return json.loads((CASE / name).read_bytes())


def check_geometry(cleanup):
    inputs = cleanup["inputs_after_close"]
    originals = CASE.parent / "multiple-cutouts-20261120"
    valid = originals / "Pocket two slots.svg"
    assert digest(valid) == inputs[valid.name]
    invalid = originals / "Overlapping slots.svg"
    assert digest(invalid) == inputs[invalid.name]
    single = CASE.parent / "native-svg-cutout-20261120" / "Pocket imported.weftforge"
    assert digest(single) == inputs["Pocket imported.weftforge"]
    geometry = check_case(
        valid, CASE / "Imported.weftforge", CASE / "Imported.obj", CASE / "Edited.weftforge",
        CASE / "Edited.obj", CASE / "Refused.weftforge", (single, CASE / "Inspector.weftforge"),
        CASE / "Inspector.obj", CASE / "InspectorRedo.weftforge",
    )
    assert geometry == read("GEOMETRY.json")
    assert (CASE / "Imported.obj").read_bytes() == (originals / "Pocket.obj").read_bytes()
    native = read("Imported.weftforge")
    adapter = json.loads((originals / "Pocket.weftforge").read_bytes())
    assert (native["project"]["revision"], adapter["project"]["revision"]) == (3, 1)
    native["project"]["revision"] = 1
    assert native == adapter


def check_wayland(session, cleanup):
    surfaces = session["surfaces"]
    studio = [surfaces[phase][0] for phase in ("import", "fresh_reopen", "add_hole")]
    assert [w["app_id"] for w in studio] == ["WeftForge"] * 3
    assert all(w["shell"] == "xdg_shell" for w in studio + surfaces["gtk_chooser"])
    pids = [w["pid"] for w in studio]
    assert len(set(pids)) == 3 and pids == cleanup["studio_pids_absent"]
    assert surfaces["after_add_redo"] == surfaces["add_hole"]
    assert {w["app_id"] for w in surfaces["gtk_chooser"]} == {"WeftForge", "xdg-desktop-portal-gtk"}
    assert cleanup["studio_exit_codes"] == [0, 0, 0] and cleanup["owned_guest_final_state"] == "Stopped"
    for path in CASE.glob("*.png"):
        header = path.read_bytes()[:24]
        assert header[:16] == bytes.fromhex("89504e470d0a1a0a0000000d49484452")
        expected = (820, 1000) if path.name.startswith("workspace-") else (1640, 1000)
        assert struct.unpack(">II", header[16:24]) == expected


def check_package(session):
    # The guest's independent audit read the actual binaries and notices.
    # Here, match its retained receipt to both release manifests and the ledger.
    result = read("PACKAGE-READBACK.json")
    package = read("OFFLINE-PACKAGE.json")
    manifest = read("manifest.json")
    entries = {}
    for line in (CASE / "PACKAGE-SHA256SUMS").read_text().splitlines():
        checksum, name = line.split("  ./", 1)
        assert name not in entries
        entries[name] = checksum
    assert package["package"] == result
    assert result["ledger_sha256"] == digest(CASE / "PACKAGE-SHA256SUMS")
    assert result["manifest_sha256"] == entries["licenses/manifest.json"] == digest(CASE / "manifest.json")
    assert len(entries) == result["verified_files"] == 443
    assert result["emitted_cargo_crates"] == len(manifest["crates"]) == 243
    assert result["unresolved_cargo_notices"] == [c["name"] for c in manifest["crates"] if c["unresolved"]] == []
    records = [*manifest["artifacts"], *manifest["project_notices"]]
    for crate in manifest["crates"]:
        records.extend(crate["files"])
    assert all(entries[r["path"]] == r["sha256"] for r in records)
    assert len({r["path"] for r in records}) == result["verified_distinct_manifest_paths"] == 441
    assert result["executables"] == {r["path"]: r["sha256"] for r in manifest["artifacts"]}
    assert session["tested_commit"] == package["tested_commit"]
    assert session["packaged_studio_sha256"] == entries["weftforge"]
    previous = CASE.parent / "debian-wayland-svg-cutout-20261120" / "manifest.json"
    assert package["earlier_package_manifest_sha256"] == digest(previous)


def main():
    session, cleanup = read("WAYLAND.json"), read("CLEANUP.json")
    check_geometry(cleanup)
    check_wayland(session, cleanup)
    check_package(session)
    print("Three boundaries measured in both exports. Material area changed from 0.0756 to 0.0744 m².")
    print("The overlap refusal preserved the edited bytes. Add hole and Redo preserved both authored rings.")
    print("Three native Wayland processes, the GTK chooser and the 443-file package ledger agree.")


if __name__ == "__main__":
    main()
