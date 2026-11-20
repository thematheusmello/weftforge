"""Verify the actual staged release files and the emitted Cargo notice manifest."""
import argparse
from pathlib import Path
import hashlib
import json


def digest(path):
    with path.open("rb") as f:
        return hashlib.file_digest(f, "sha256").hexdigest()


def audit(root, ledger):
    entries = {}
    for line in ledger.read_text().splitlines():
        checksum, name = line.split("  ./", 1)
        assert name not in entries and not Path(name).is_absolute() and ".." not in Path(name).parts
        entries[name] = checksum
    files = {p.relative_to(root).as_posix(): p for p in root.rglob("*") if p.is_file()}
    assert entries.keys() == files.keys()
    for name, path in files.items():
        assert not path.is_symlink() and digest(path) == entries[name]
    manifest = json.loads((root / "licenses/manifest.json").read_bytes())
    assert manifest["target"] == "aarch64-unknown-linux-gnu"
    records = [*manifest["artifacts"], *manifest["project_notices"]]
    for crate in manifest["crates"]:
        records.extend(crate["files"])
    for record in records:
        assert entries[record["path"]] == record["sha256"]
        assert files[record["path"]].stat().st_size == record["size_bytes"]
    return {"target": manifest["target"], "verified_files": len(files),
            "verified_distinct_manifest_paths": len({record["path"] for record in records}),
            "emitted_cargo_crates": len(manifest["crates"]),
            "unresolved_cargo_notices": [dict(name=c["name"], version=c["version"])
                                          for c in manifest["crates"] if c["unresolved"]],
            "executables": {record["path"]: record["sha256"] for record in manifest["artifacts"]},
            "manifest_sha256": entries["licenses/manifest.json"], "ledger_sha256": digest(ledger)}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("package", type=Path)
    parser.add_argument("ledger", type=Path)
    args = parser.parse_args()
    print(json.dumps(audit(args.package, args.ledger), indent=2))
