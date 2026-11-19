#!/usr/bin/env python3
"""Validate a generated Apple Silicon WeftForge package from its own manifest."""

from hashlib import file_digest, sha256
from pathlib import Path, PurePosixPath
import json
import plistlib
import subprocess
import sys


def verify_file(package, record):
    path = PurePosixPath(record["path"])
    assert str(path) == record["path"] and not path.is_absolute() and ".." not in path.parts
    file = package.joinpath(*path.parts)
    assert file.is_file() and not file.is_symlink()
    assert file.resolve().is_relative_to(package)
    assert file.stat().st_size == record["size_bytes"]
    with file.open("rb") as handle:
        assert file_digest(handle, "sha256").hexdigest() == record["sha256"]
    return path


def check(package):
    raw = (package / "licenses/manifest.json").read_bytes()
    manifest = json.loads(raw)
    assert manifest["version"] == 1 and manifest["target"] == "aarch64-apple-darwin"
    app = package / "WeftForge.app/Contents"
    bundle = (app / "Info.plist").read_bytes()
    plist = plistlib.loads(bundle)
    assert plist["CFBundleIdentifier"] == "org.weftforge.studio"
    assert plist["CFBundleExecutable"] == "weftforge"
    root_paths = {
        verify_file(package, item) for item in [*manifest["artifacts"], *manifest["project_notices"]]
    }
    assert root_paths == set(map(PurePosixPath, (
        "weftforge", "weftforge-cli", "WeftForge.app/Contents/MacOS/weftforge",
        "LICENSE-MIT", "LICENSE-APACHE", "THIRD_PARTY.md", "licenses/OFL-Inter.txt", "licenses/OFL-JetBrainsMono.txt",
    )))
    assert manifest["artifacts"][0]["sha256"] == manifest["artifacts"][2]["sha256"]
    for bin_name in ("weftforge", "weftforge-cli"):
        assert "Mach-O 64-bit executable arm64" in subprocess.check_output(["file", "-b", package / bin_name], text=True)
    for project_notice in manifest["project_notices"]:
        path = PurePosixPath(project_notice["path"])
        bundle_path = app / "Resources/licenses" / path.name
        assert (package / path).read_bytes() == bundle_path.read_bytes()
    assert not (app / "Resources/licenses/manifest.json").exists()
    assert not (package.parent / ".package-staging").exists()

    notices = set()
    missing = []
    for item in manifest["crates"]:
        label = f"{item['name']}@{item['version']}"
        if item["unresolved"]:
            missing.append(label)
        for record in item["files"]:
            relative = verify_file(package, record)
            assert relative.parts[:3] == ("licenses", "cargo", f"{item['name']}-{item['version']}")
            assert relative not in notices
            notices.add(relative)
            standalone = app / "Resources" / relative
            assert standalone.read_bytes() == (package / relative).read_bytes()
    assert {p.as_posix() for p in notices} == {
        p.relative_to(package).as_posix() for p in (package / "licenses/cargo").rglob("*") if p.is_file()
    }
    assert len({(x["name"], x["version"], x["source"]) for x in manifest["crates"]}) == len(manifest["crates"])
    return {
        "target": manifest["target"],
        "manifest_sha256": sha256(raw).hexdigest(),
        "executables": manifest["artifacts"],
        "verified_cargo_notice_files": len(notices),
        "emitted_third_party_libraries": len(manifest["crates"]),
        "unresolved_library_notices": sorted(missing),
        "standalone_copied_notice_bytes_exact": True,
    }


if __name__ == "__main__":
    print(json.dumps(check(Path(sys.argv[1]).resolve()), indent=2))
