"""Bind the existing full-source protocol to clean F; never launch a product check."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parent
WT = ROOT.parent.parent
OLD = ROOT.parent / "contact-recovery-integrated-source-002-20261118"
F = "c766f3fa48c03ac698da8b22cc3bf927ee71f195"


def sha(path):
    digest = hashlib.sha256()
    with Path(path).open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def write(name, value):
    with (ROOT / name).open("x") as stream:
        json.dump(value, stream, indent=2, sort_keys=True, allow_nan=False)
        stream.write("\n")


def git(*argv):
    return subprocess.check_output(["/usr/bin/git", *argv], cwd=WT, text=True).strip()


assert sys.dont_write_bytecode and sys.flags.optimize == 0
assert git("rev-parse", "HEAD") == F
assert git("status", "--porcelain=v1", "--untracked-files=no") == ""
old = json.loads((OLD / "full-001.json").read_text())
assert len(old["source91"]) == 91 and len(old["locked_registry_archives"]) == 410
assert len(old["cached_registry_trees"]) == 410 and len(old["commands"]) == 4
assert all(Path(name).name == name for name in old["controller_inputs"])
assert all(Path(name).is_absolute() for name in old["config_inputs"])
for name, expected in old["controller_inputs"].items():
    assert sha(OLD / name) == expected
    shutil.copyfile(OLD / name, ROOT / name)

plan = dict(old)
plan.update(
    actual_authorization="Proposed one fresh complete fmt/workspace480tests/strictClippy/native+CLIdebugbuild on clean F; explicit root GO is required before launch. No GUI/model/xtask/generator/network/push.",
    case="WF-20261118-CAPSULE-SUPPORTING-PLANE-INTEGRATED-SOURCE",
    head=F,
    tracked_status="",
    current_source_composition="Reviewed002 capsule-only N1/N3 current-side supporting-plane contact, shared scaled point geometry and fail-closed diagnostics; unchanged prior physics/CLI/native source plus current physics.md and STATUS.",
    expected_count_derivation="472 prior complete-workspace tests plus eight new simulator tests (94 to 102) = 480; all other package counts unchanged; 32 summaries.",
    bounds_qualification="Same integration checkout's own warm target/cache only; no compiled target copies. Original shared900s/aggregate1.5GiB/output8MiB/free-disk8GB limits and sampling/error/group-cleanup policy.",
    root_GO_recorded_utc=None,
    no_owned_GUI_or_model_running_at_admission=True,
    cold_target_expected=not (WT / "target").exists(),
    all_earlier_cases_read_only=True,
)
plan.pop("prior_attempt", None)
tracked = set(git("ls-files").splitlines())
assert set(old["source91"]) <= tracked
code = {name for name in tracked if name.startswith(("apps/", "crates/", "xtask/", "vendor/")) and (name.endswith(".rs") or name.endswith("/Cargo.toml"))}
assert code <= set(old["source91"])
plan["source91"] = {}
for name in old["source91"]:
    path = WT / name
    assert path.is_file() and not path.is_symlink()
    plan["source91"][name] = {"sha256": sha(path), "bytes": path.stat().st_size, "mode": path.stat().st_mode & 0o777}
expected_sources = {
    "crates/weftforge-sim/src/contact.rs": "89d3d5f31dc2d6cb7ce5e207c070bb774aeb90ed36df50b9d0acb34fd2a8cac4",
    "crates/weftforge-sim/src/capsule.rs": "ac833840d7568fd2deda6731325d4c33f29e1935df386bfeec7d1f206dd4e322",
    "docs/physics.md": "628953bb7294bc373fca731937b963cd2b2b3615b0c67085d14a5092ffadc6a9",
}
assert all(plan["source91"][name]["sha256"] == digest for name, digest in expected_sources.items())
plan["config_inputs"] = {name: sha(name) if Path(name).is_file() else None for name in old["config_inputs"]}
plan["controller_inputs"] = {name: sha(ROOT / name) for name in old["controller_inputs"]}
plan["commands"] = json.loads(json.dumps(old["commands"]))
plan["commands"][1]["expected_tests"] = {"passed": 480, "failed": 0, "ignored": 0, "summaries": 32}
assert all(os.environ.get(key) in (None, "") for key in plan["rejected_ambient_keys"])
assert all(sha(tool["path"]) == tool["sha256"] for tool in plan["tools"].values())
assert all(sha(path) == digest for path, digest in plan["locked_registry_archives"].items())
sys.path.insert(0, str(ROOT))
from runfull import cached_registry_tree_digest
assert cached_registry_tree_digest(plan["cached_registry_trees"]) == plan["cached_registry_tree_digest"]
assert plan["child_env"]["CARGO_TARGET_DIR"] == str(WT / "target")
assert plan["child_env"]["CARGO_BUILD_JOBS"] == "1" and plan["child_env"]["CARGO_NET_OFFLINE"] == "true"
assert plan["cold_target_expected"] is False
assert shutil.disk_usage(WT).free >= 8_000_000_000
review = WT.parent.parent / ".local/capsule-supporting-plane-source-review-20261118/REVIEW-002.md"
assert sha(review) == "dea35d3285566f8b226978618f767d06d043f1022bd77a51cb8ee301941a7f66"
plan["independent_review"] = {"path": str(review), "sha256": sha(review), "disposition": "SOURCE_READY"}
preserved = [
    OLD,
    ROOT.parent / "contact-recovery-integrated-source-20261118",
    WT.parent / "neckline-coupling-regression-20261118/.local/runs",
    ROOT.parent / "harbor-contact-recovery-preparation-20261118/runs/base",
]
manifest = {str(path): sha(path) for case in preserved for path in sorted(case.rglob("*")) if path.is_file()}
write("PRIOR-ARTIFACTS.json", manifest)
plan["preservation_manifest"] = {"path": "PRIOR-ARTIFACTS.json", "sha256": sha(ROOT / "PRIOR-ARTIFACTS.json"), "files": len(manifest)}
write("full-001.json", plan)
go = {"metadata_commit": F, "pinned_env": plan["child_env"], "plan": "full-001.json", "plan_sha256": sha(ROOT / "full-001.json")}
write("ROOT-GO.json", go)
proposal = {"status": "prepared_not_launched_explicit_root_GO_pending", "head": F, "source91_count": 91, "source91_roster": list(plan["source91"]), "source91_changed_from_prior": [name for name, desc in plan["source91"].items() if desc != old["source91"][name]], "config_inputs": plan["config_inputs"], "child_env": plan["child_env"], "commands": plan["commands"], "tools": plan["tools"], "caps": plan["caps"], "wall_seconds": plan["wall_seconds"], "cold_target_expected": plan["cold_target_expected"], "cached_registry_tree_digest": plan["cached_registry_tree_digest"], "locked_registry_archive_count": 410, "preservation_manifest_files": len(manifest), "available_bytes": shutil.disk_usage(WT).free, "helper_hashes": plan["controller_inputs"], "plan_sha256": sha(ROOT / "full-001.json"), "GO_sha256": sha(ROOT / "ROOT-GO.json"), "prepare_sha256": sha(Path(__file__))}
write("PROPOSAL.json", proposal)
print(json.dumps({key: value for key, value in proposal.items() if key not in ("source91_roster", "config_inputs", "child_env", "commands", "tools")}, sort_keys=True))
