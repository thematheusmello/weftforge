"""Audit two offline Apple Silicon package manifests and read both shipped notice copies."""
import hashlib
import json
from pathlib import Path
import sys
import tarfile
import tomllib

FRAMEWORKS = {
    "objc2-app-kit", "objc2-core-foundation", "objc2-core-graphics", "objc2-metal",
    "objc2-quartz-core", "objc2-uniform-type-identifiers",
}
APACHE = "cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30"
APACHE_URL = "https://www.apache.org/licenses/LICENSE-2.0"
FRAMEWORK_REV = "7b1abfd750a2cacaea71d6a56ecfb83cb7de560b"
DISPATCH2_REV = "8852b424193ca41602281b3d7540d7c8ed51e49a"
DECLARATION = "7f976f7e9cb2d87df7230606feb932c3f21ac0e664045a775b600046ff850c54"
OPEN = {"block2@0.6.2", "dispatch@0.2.0", "objc2@0.6.5", "objc2-encode@4.1.0", "objc2-foundation@0.3.2"}
SELECTED = {f"{name}@0.3.2" for name in FRAMEWORKS} | {"dispatch2@0.3.1"}


def read_checked(root, record):
    file = root / record["path"]
    assert root in file.resolve().parents and not file.is_symlink()
    data = file.read_bytes()
    assert len(data) == record["size_bytes"] and hashlib.sha256(data).hexdigest() == record["sha256"], file
    return data


def audit(before, package, source, cache):
    manifest = package / "licenses/manifest.json"
    old, new = json.loads(before.read_bytes()), json.loads(manifest.read_bytes())
    assert new["target"] == "aarch64-apple-darwin"
    assert {k: v for k, v in old.items() if k != "crates"} == {k: v for k, v in new.items() if k != "crates"}
    resources = package / "WeftForge.app/Contents/Resources"
    for record in new["artifacts"]:
        read_checked(package, record)
    for record in new["project_notices"]:
        assert read_checked(package, record) == (resources / "licenses" / Path(record["path"]).name).read_bytes()

    locked = {(p["name"], p["version"]): p.get("checksum")
              for p in tomllib.loads((source / "Cargo.lock").read_text())["package"]}
    old_rows = {f"{c['name']}@{c['version']}": c for c in old["crates"]}
    assert len(old_rows) == len(old["crates"]) == len(new["crates"])
    assert {key for key, row in old_rows.items() if row["unresolved"]} == SELECTED | OPEN
    encountered = set()
    notices = set()
    archive_proofs = {}
    for row in new["crates"]:
        name, version = row["name"], row["version"]
        key = f"{name}@{version}"
        previous = old_rows[key]
        assert key not in encountered
        encountered.add(key)
        assert bool(row["unresolved"]) == (key in OPEN)
        if key in SELECTED:
            assert row["declared_license"] == "Zlib OR Apache-2.0 OR MIT"
            assert previous["unresolved"] and not row["unresolved"]
            assert {k:v for k,v in row.items() if k not in ("files","unresolved")} == {
                k:v for k,v in previous.items() if k not in ("files","unresolved")
            }
            declaration, apache = sorted(row["files"], key=lambda r: r["upstream"]["kind"], reverse=True)
            assert previous["files"] == [declaration]
            revision = FRAMEWORK_REV if name in FRAMEWORKS else DISPATCH2_REV
            assert declaration["sha256"] == DECLARATION
            assert declaration["upstream"] == {"name":"LICENSE.md", "kind":"license_declaration",
                "url":f"https://raw.githubusercontent.com/madsmtm/objc2/{revision}/LICENSE.md"}
            assert apache["sha256"] == APACHE and apache["size_bytes"] == 11358
            assert apache["upstream"] == {"name":"LICENSE-APACHE.txt", "kind":"code_license", "url":APACHE_URL}
            assert APACHE_URL.encode() in read_checked(package, declaration)
            assert read_checked(package, apache) == (source / "vendor/cargo-notices/texts" / f"{APACHE}.txt").read_bytes()
            archive, = cache.glob(f"*/{name}-{version}.crate")
            digest = hashlib.sha256(archive.read_bytes()).hexdigest()
            assert locked[name, version] == digest
            with tarfile.open(archive) as contents:
                vcs = json.load(contents.extractfile(f"{name}-{version}/.cargo_vcs_info.json"))
                assert vcs["git"]["sha1"] == revision
                cargo = tomllib.loads(contents.extractfile(f"{name}-{version}/Cargo.toml").read().decode())
                assert cargo["package"]["license"] == row["declared_license"]
                archive_proofs[key] = {"archive_sha256": digest, "revision": revision, "crate_path": vcs["path_in_vcs"]}
        else:
            assert row == previous, key
        for record in row["files"]:
            assert read_checked(package, record) == read_checked(resources, record)
            notices.add(record["path"])
    assert encountered == old_rows.keys()
    for root in (package, resources):
        assert notices == {p.relative_to(root).as_posix() for p in (root / "licenses/cargo").rglob("*") if p.is_file()}
    return {
        "target": new["target"], "emitted_crates": len(encountered), "cargo_notice_files_in_each_copy": len(notices),
        "project_notices_verified_in_each_copy": len(new["project_notices"]), "executable_records_verified": new["artifacts"],
        "previous_unresolved": len(OPEN | SELECTED), "resolved": sorted(SELECTED), "unresolved": sorted(OPEN),
        "locked_archives": archive_proofs, "before_sha256": hashlib.sha256(before.read_bytes()).hexdigest(),
        "after_sha256": hashlib.sha256(manifest.read_bytes()).hexdigest(),
    }


if __name__ == "__main__":
    before, package, source, cache = (Path(arg).resolve() for arg in sys.argv[1:])
    print(json.dumps(audit(before, package, source, cache), indent=2))
