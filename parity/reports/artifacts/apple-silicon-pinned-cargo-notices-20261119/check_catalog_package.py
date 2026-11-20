#!/usr/bin/env python3
"""Compare the current package to the prior emitted graph and the checked-in upstream texts."""

from hashlib import sha256
import json
from pathlib import Path, PurePosixPath
import sys


CRATES_IO = "registry+https://github.com/rust-lang/crates.io-index"


def digest(data):
    return sha256(data).hexdigest()


def verify_file(package, record):
    relative = PurePosixPath(record["path"])
    assert not relative.is_absolute() and ".." not in relative.parts and relative.as_posix() == record["path"]
    file = package.joinpath(*relative.parts)
    assert file.resolve().is_relative_to(package) and not file.is_symlink()
    data = file.read_bytes()
    assert len(data) == record["size_bytes"] and digest(data) == record["sha256"]
    return relative, data


def check(baseline, package, catalog_path):
    old_raw = baseline.read_bytes()
    new_raw = (package / "licenses/manifest.json").read_bytes()
    old, new = map(json.loads, (old_raw, new_raw))
    assert old["target"] == new["target"] == "aarch64-apple-darwin"
    assert old["artifacts"] == new["artifacts"]
    resources = package / "WeftForge.app/Contents/Resources"
    for record in new["artifacts"]:
        verify_file(package, record)
    for record in new["project_notices"]:
        relative, contents = verify_file(package, record)
        assert (resources / "licenses" / relative.name).read_bytes() == contents

    catalog = json.loads(catalog_path.read_bytes())
    pins = {
        (crate["name"], crate["version"]): group
        for group in catalog for crate in group["crates"]
    }
    assert len(pins) == 29
    old_by_key = {(row["name"], row["version"]): row for row in old["crates"]}
    current_keys = {(row["name"], row["version"]) for row in new["crates"]}
    assert current_keys == old_by_key.keys() and len(current_keys) == len(new["crates"])
    assert pins.keys() <= current_keys
    old_missing = {key for key, row in old_by_key.items() if row["unresolved"]}
    resolved, unresolved, declaration_only = set(), set(), set()
    unique_files, unique_pins = set(), set()
    pinned_text_count = 0
    for row in new["crates"]:
        key = row["name"], row["version"]
        assert row["source"] == old_by_key[key]["source"]
        assert [f for f in row["files"] if "upstream" not in f] == old_by_key[key]["files"]
        as_recorded = {f["path"]: f for f in row["files"] if "upstream" in f}
        assert (key in pins) == bool(as_recorded)
        if key in pins:
            assert row["source"] == CRATES_IO
            group = pins[key]
            mapped = set()
            for document in group["documents"]:
                key_path = f'licenses/cargo/upstream/{group["project"]}/{document["sha256"]}/{document["name"]}'
                record = as_recorded[key_path]
                assert record["upstream"] == {k: document[k] for k in ("kind", "name", "url")}
                assert (record["size_bytes"], record["sha256"]) == (document["size_bytes"], document["sha256"])
                relative, contents = verify_file(package, record)
                assert (catalog_path.parent / "texts" / f'{document["sha256"]}.txt').read_bytes() == contents
                unique_pins.add(relative.as_posix())
                mapped.add(key_path)
                pinned_text_count += 1
            assert mapped == as_recorded.keys()
            if any(f["upstream"]["kind"] == "code_license" for f in as_recorded.values()):
                assert not row["unresolved"]
                resolved.add(key)
            else:
                assert row["unresolved"]
                declaration_only.add(key)
        if row["unresolved"]:
            unresolved.add(key)
        for record in row["files"]:
            relative, contents = verify_file(package, record)
            assert (resources / relative).read_bytes() == contents
            unique_files.add(relative.as_posix())

    assert resolved <= old_missing and unresolved == old_missing - resolved
    assert (len(current_keys), len(old_missing), len(resolved), len(declaration_only), len(unresolved)) == (165, 31, 18, 11, 13)
    assert {p.relative_to(package).as_posix() for p in (package / "licenses/cargo").rglob("*") if p.is_file()} == unique_files
    assert {p.relative_to(resources).as_posix() for p in (resources / "licenses/cargo").rglob("*") if p.is_file()} == unique_files
    assert not (package.parent / ".package-staging").exists()

    return {
        "target": new["target"],
        "baseline_manifest_sha256": digest(old_raw),
        "manifest_sha256": digest(new_raw),
        "executables": new["artifacts"],
        "emitted_third_party_libraries": len(current_keys),
        "verified_cargo_notice_files": len(unique_files),
        "standalone_notice_bytes_exact": True,
        "unique_pinned_package_paths": len(unique_pins),
        "pinned_text_references": pinned_text_count,
        "resolved_code_texts": ["@".join(x) for x in sorted(resolved)],
        "declaration_only": ["@".join(x) for x in sorted(declaration_only)],
        "unresolved_notices": ["@".join(x) for x in sorted(unresolved)],
    }


if __name__ == "__main__":
    baseline, package, catalog = map(Path, sys.argv[1:])
    print(json.dumps(check(baseline, package.resolve(), catalog), indent=2))
