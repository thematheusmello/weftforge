"""Audit the Debian package ledger and Cargo notices, with optional live file readback."""
import argparse
import hashlib
import json
from pathlib import Path
import tarfile
import tomllib


def sha(data):
    return hashlib.sha256(data).hexdigest()


def package_files(ledger):
    entries = {}
    for line in ledger.read_text().splitlines():
        digest, name = line.split("  ./", 1)
        path = Path(name)
        assert not path.is_absolute() and ".." not in path.parts and name not in entries
        assert len(digest) == 64 and all(c in "0123456789abcdef" for c in digest)
        entries[name] = digest
    return entries


def crates_by_identity(manifest):
    crates = {(row["name"], row["version"]): row for row in manifest["crates"]}
    assert len(crates) == len(manifest["crates"])
    return crates


def audit(args):
    before = json.loads(args.baseline.read_bytes())
    after = json.loads(args.manifest.read_bytes())
    assert before["target"] == after["target"] == "aarch64-unknown-linux-gnu"
    prior, current = crates_by_identity(before), crates_by_identity(after)
    assert prior.keys() == current.keys() and len(current) == 234
    old_open = {key for key, row in prior.items() if row["unresolved"]}
    assert {name for name, _ in old_open} == {
        "accesskit_atspi_common", "accesskit_consumer", "accesskit_unix",
        "gpu-descriptor", "gpu-descriptor-types", "hexf-parse", "spirv",
    }

    ledger = package_files(args.ledger)
    assert ledger["licenses/manifest.json"] == sha(args.manifest.read_bytes())
    records = [*after["artifacts"], *after["project_notices"]]
    for row in current.values():
        records.extend(row["files"])
    sizes = {}
    for record in records:
        name = record["path"]
        assert record["sha256"] == ledger[name]
        assert sizes.setdefault(name, record["size_bytes"]) == record["size_bytes"]
    if args.package:
        files = {p.relative_to(args.package).as_posix(): p
                 for p in args.package.rglob("*") if p.is_file()}
        assert files.keys() == ledger.keys()
        for name, path in files.items():
            assert not path.is_symlink() and sha(path.read_bytes()) == ledger[name]
            if name in sizes:
                assert path.stat().st_size == sizes[name]

    source = args.root / "vendor/cargo-notices"
    groups = json.loads((source / "index.json").read_bytes())
    pins = {(pin["name"], pin["version"]): (pin, group)
            for group in groups for pin in group["crates"]}
    lock = tomllib.loads((args.root / "Cargo.lock").read_text())
    checksums = {(p["name"], p["version"]): p["checksum"]
                 for p in lock["package"] if "checksum" in p}
    selected, selected_files = {}, set()
    for key in sorted(old_open):
        name, version = key
        row = current[key]
        assert row["source"] == prior[key]["source"] and row["declared_license"] == prior[key]["declared_license"]
        pin, group = pins[key]
        assert checksums[key] == pin["lock_sha256"]
        if args.cache:
            archives = list(args.cache.glob(f"*/{name}-{version}.crate"))
            assert len(archives) == 1 and sha(archives[0].read_bytes()) == checksums[key]
            with tarfile.open(archives[0]) as archive:
                info = archive.extractfile(f"{name}-{version}/.cargo_vcs_info.json").read()
                assert sha(info) == pin["vcs_info_sha256"]
        documents = {doc["name"]: doc for doc in group["documents"]}
        staged = {f["upstream"]["name"]: f for f in row["files"] if "upstream" in f}
        assert documents.keys() == staged.keys()
        assert any(d["kind"] == "code_license" for d in documents.values())
        for filename, doc in documents.items():
            record = staged[filename]
            assert record["upstream"] == {k: doc[k] for k in ("name", "kind", "url")}
            assert record["sha256"] == doc["sha256"] and record["size_bytes"] == doc["size_bytes"]
            text = (source / "texts" / (doc["sha256"] + ".txt")).read_bytes()
            assert sha(text) == doc["sha256"] and len(text) == doc["size_bytes"]
            selected_files.add(record["path"])
        selected[name] = {"version": version, "declared_license": row["declared_license"],
                          "archive_sha256": checksums[key], "documents": sorted(documents)}
    unresolved = [row["name"] for row in current.values() if row["unresolved"]]
    assert not unresolved
    return {"target": after["target"], "manifest_sha256": ledger["licenses/manifest.json"],
            "ledger_files": len(ledger), "manifest_file_paths": len(sizes),
            "emitted_crates": len(current), "previously_unresolved": len(old_open),
            "unresolved": unresolved, "covered_crates": selected,
            "selected_notice_paths": sorted(selected_files),
            "artifacts": {a["path"]: a["sha256"] for a in after["artifacts"]}}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    here = Path(__file__).resolve().parent
    parser.add_argument("--root", type=Path, default=here.parents[3])
    parser.add_argument("--baseline", type=Path, default=here.parent / "debian-native-save-jpeg-20261120/manifest.json")
    parser.add_argument("--manifest", type=Path, default=here / "manifest.json")
    parser.add_argument("--ledger", type=Path, default=here / "PACKAGE-SHA256SUMS")
    parser.add_argument("--package", type=Path)
    parser.add_argument("--cache", type=Path)
    print(json.dumps(audit(parser.parse_args()), indent=2))
