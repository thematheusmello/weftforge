"""Preparation and authorization bindings only; no dynamics or numerical oracle."""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import stat
import struct
import sys
import time

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[1]
PROTOCOL_SHA = "83944a61756d5df0eab2656c87cd0626cabc3cc4968a6cd4b4eec770fa853591"
SOURCE_BINDING_SHA = "655caa99716c35577d4964ee977aa6705acdafe607d824b35179d929e9d30f02"
GATES_BINDING_SHA = "7b9350148358b93b3731166eef29e824e2c1f42aaf25aa25bba08ac33220facf"


def require(condition, message):
    if not condition:
        raise ValueError(message)


def deadline_check(deadline):
    require(deadline is None or time.monotonic() < deadline, "Whole-case deadline closed; no numerical continuation")


def read(path):
    def pairs(items):
        value = {}
        for key, item in items:
            require(key not in value, "Duplicate external JSON key")
            value[key] = item
        return value
    def floating(token):
        value = float(token)
        require(math.isfinite(value), "Nonfinite external JSON number")
        return value
    def integer(token):
        value = int(token)
        try:
            finite = math.isfinite(value)
        except OverflowError:
            finite = False
        require(finite, "External JSON integer outside finite range")
        return value
    return json.loads(Path(path).read_text(), object_pairs_hook=pairs, parse_float=floating,
                      parse_int=integer, parse_constant=lambda token: require(False, "Nonfinite external JSON token"))


def safe_file(value, base):
    path, base = Path(value), Path(base).resolve()
    require(".." not in path.parts, "Parent traversal rejected")
    if not path.is_absolute():
        path = base / path
    require(path.absolute().is_relative_to(base), "Artifact escaped bound directory")
    current = path
    while current != base:
        require(not current.is_symlink(), "Symlink artifact rejected")
        current = current.parent
    require(path.is_file() and stat.S_ISREG(path.stat().st_mode), "Missing regular artifact: " + path.name)
    return path


def sha(path, deadline=None):
    digest = hashlib.sha256()
    with Path(path).open("rb") as stream:
        while True:
            deadline_check(deadline)
            block = stream.read(1024 * 1024)
            if not block:
                break
            digest.update(block)
    deadline_check(deadline)
    return digest.hexdigest()


def files_below(directory):
    directory = Path(directory)
    require(directory.is_dir() and not directory.is_symlink(), "Missing real directory: " + directory.name)
    result = []
    for path in sorted(directory.iterdir()):
        mode = path.lstat().st_mode
        require(not stat.S_ISLNK(mode), "Symlink directory member rejected")
        if stat.S_ISDIR(mode):
            result.extend(files_below(path))
        else:
            require(stat.S_ISREG(mode), "Nonregular directory member rejected")
            result.append(path)
    return result


def descriptor(value, expected=None, deadline=None):
    require(isinstance(value, dict) and set(value) == {"path", "sha256"}, "Descriptor must contain path/sha256")
    require(isinstance(value["path"], str) and not Path(value["path"]).is_absolute(), "Descriptor must be repository-relative")
    path = safe_file(value["path"], REPO)
    require(sha(path, deadline) == value["sha256"], "Artifact hash mismatch: " + path.name)
    if expected is not None:
        require(path == expected, "Wrong actual execution wrapper" if expected.name == "runner.py" else "Wrong bound artifact: " + expected.name)
    return path


def same(a, b):
    if type(a) is float and type(b) is float:
        return struct.pack(">d", a) == struct.pack(">d", b)
    if isinstance(a, list) and isinstance(b, list):
        return len(a) == len(b) and all(same(x, y) for x, y in zip(a, b))
    if isinstance(a, dict) and isinstance(b, dict):
        return a.keys() == b.keys() and all(same(a[k], b[k]) for k in a)
    return type(a) is type(b) and a == b


def pinned_source():
    path = safe_file("SOURCE-BINDING.json", ROOT)
    require(sha(path) == SOURCE_BINDING_SHA, "Exact source binding changed")
    value = read(path)
    require(set(value) == {"schema", "kind", "source_commit", "source_hashes"}
            and type(value["schema"]) is int and value["schema"] == 1
            and value["kind"] == "Exact authoritative integrated runtime binding", "Wrong source binding schema")
    commit = value["source_commit"]
    require(isinstance(commit, str) and len(commit) == 40 and all(c in "0123456789abcdef" for c in commit), "Source commit not finalized")
    require(commit != "0770204a54abdf5f929b6626b01d7fcfb0e13ca4", "Old closed source cannot be rerun as new case")
    require(isinstance(value["source_hashes"], dict) and value["source_hashes"], "Source hashes not finalized")
    return value


SOURCE = pinned_source()["source_commit"]


def verify_readonly_inputs(deadline=None):
    value = read(safe_file("READONLY-INPUT-BINDING.json", ROOT))
    for name, digest in value["files"].items():
        require(sha(safe_file(name, REPO), deadline) == digest, "Immutable prior evidence changed: " + Path(name).name)
    return {"files": len(value["files"]), "all_hashes_exact": True}


def verify_static(*, require_freeze=True, deadline=None):
    deadline_check(deadline)
    require(sys.platform == "darwin" and hasattr(os, "wait4"), "Exact producer RSS requires Darwin wait4 bytes")
    require(sha(safe_file("PROTOCOL.json", ROOT), deadline) == PROTOCOL_SHA, "Prospective protocol changed")
    protocol, source = read(ROOT / "PROTOCOL.json"), pinned_source()
    require(sys.implementation.name == protocol["python"]["implementation"]
            and list(sys.version_info[:3]) == protocol["python"]["version"]
            and same(sum([1.0, 1e-16, 1e-16]), protocol["python"]["builtin_sum_probe"]), "Frozen Python/norm arithmetic environment changed")
    require(set(source["source_hashes"]) == set(protocol["source_file_paths"]) and len(source["source_hashes"]) == 45, "Frozen runtime path inventory differs")
    changed = sorted(name for name, digest in source["source_hashes"].items() if protocol["baseline_runtime_hashes"].get(name) != digest)
    require(changed == protocol["allowed_changed_runtime_paths"], "Runtime change escaped reviewed three-file source delta")
    pretest = read(descriptor(protocol["authoritative_source_pretest"], deadline=deadline))
    require(pretest["source_commit"] == SOURCE and pretest["source_hashes"] == source["source_hashes"], "Authoritative source receipt differs")
    for name, digest in protocol["inputs_sha256"].items():
        require(sha(safe_file(name, ROOT), deadline) == digest, "Original fixture bytes changed")
    prior = verify_readonly_inputs(deadline)
    if require_freeze:
        freeze = read(safe_file("PREPARATION-FREEZE.json", ROOT))
        require(type(freeze["schema"]) is int and freeze["schema"] == 1
                and freeze["status"] == "frozen_prospective_preparation_no_dynamics"
                and freeze["source_commit"] == SOURCE and freeze["new_garment_execution"] is False, "Preparation not frozen")
        for name, digest in freeze["files"].items():
            require(sha(safe_file(name, ROOT), deadline) == digest, "Frozen preparation changed: " + name)
        actual_python = {p.relative_to(ROOT).as_posix() for p in files_below(ROOT) if p.suffix in (".py", ".pyc")}
        require(actual_python <= set(freeze["files"]), "Unfrozen Python source/cache in preparation")
    deadline_check(deadline)
    return {"protocol_sha256": PROTOCOL_SHA, "source_commit": SOURCE, "source_files": 45,
            "source_binding_sha256": SOURCE_BINDING_SHA, "all_hashes_exact": True, "prior_immutable_binding": prior}


def verify_gates(deadline=None):
    path = safe_file("GATES-BINDING.json", ROOT)
    require(sha(path, deadline) == GATES_BINDING_SHA, "Integrated gate binding changed")
    bound = read(path)
    require(type(bound["schema"]) is int and bound["schema"] == 1 and bound["source_commit"] == SOURCE, "Wrong integrated gate scope")
    source = read(descriptor(bound["source_pretest"], deadline=deadline))
    package = read(descriptor(bound["package_binding"], deadline=deadline))
    jobs = read(descriptor(bound["jobs"], deadline=deadline))
    terminal = read(descriptor(bound["terminal_job"], deadline=deadline))
    require(source["source_commit"] == SOURCE and source["source_hashes"] == pinned_source()["source_hashes"], "Gate source bytes differ")
    require(package["source_commit"] == SOURCE and package["source_pretest_sha256"] == bound["source_pretest"]["sha256"]
            and package["all_five_gates_exit_zero"] is True and package["git_clean"] is True, "Integrated/package gates incomplete")
    require(terminal["source_commit"] == SOURCE and terminal["status"] == "completed" and terminal["all_exit_zero"] is True, "Integrated gate wrapper not terminal success")
    commands = {"fmt": ["cargo", "fmt", "--all", "--check"], "package": ["cargo", "xtask", "package"],
                "tests": ["cargo", "test", "--workspace"],
                "clippy": ["cargo", "clippy", "--workspace", "--all-targets", "--", "-D", "warnings"],
                "xtask": ["cargo", "xtask", "verify"]}
    require(len(jobs) == 5 and {j["name"] for j in jobs} == set(commands) == set(bound["logs"]), "Wrong integrated gate inventory")
    for job in jobs:
        require(job["source_commit"] == SOURCE and job["status"] == "completed"
                and type(job["exit_code"]) is int and job["exit_code"] == 0
                and job["command"] == commands[job["name"]], "Integrated gate failed or command changed")
        descriptor(bound["logs"][job["name"]], deadline=deadline)
    descriptor(bound["native"], deadline=deadline)
    descriptor(bound["cli"], deadline=deadline)
    require(bound["native"]["sha256"] == package["binaries"]["target/release/weftforge"]
            and bound["cli"]["sha256"] == package["binaries"]["target/release/weftforge-cli"], "Actual gated native/CLI bytes differ")
    return bound


def verify_preparation_scope(auth):
    require(type(auth["schema"]) is int and auth["schema"] == 1
            and auth["status"] == "authorized_preparation_only" and auth["explicit_root_preparation_authorization"] is True,
            "Separate explicit root build/geometry preparation authorization absent")
    require(auth["source_commit"] == SOURCE and auth["cases"] == ["base", "height"]
            and auth["locked_driver_build_authorized"] is True
            and type(auth["geometry_only_executions"]) is int and auth["geometry_only_executions"] == 2
            and type(auth["physics_initializations_or_steps"]) is int and auth["physics_initializations_or_steps"] == 0
            and auth["dynamics_authorized"] is False, "Preparation authority must exclude all dynamics")


def verify_preparation(deadline=None):
    summary = verify_static(deadline=deadline)
    verify_gates(deadline)
    source = pinned_source()
    inventory = {p.relative_to(ROOT / "source").as_posix() for p in files_below(ROOT / "source")}
    require(inventory == set(source["source_hashes"]), "Producer source snapshot inventory differs")
    for name, digest in source["source_hashes"].items():
        require(sha(safe_file(name, ROOT / "source"), deadline) == digest, "Producer source snapshot bytes differ")
    path = safe_file("PRODUCER-BINDING.json", ROOT)
    producer = read(path)
    require(type(producer["schema"]) is int and producer["schema"] == 1 and producer["source_commit"] == SOURCE
            and producer["protocol_sha256"] == PROTOCOL_SHA and producer["source_binding_sha256"] == SOURCE_BINDING_SHA
            and producer["preparation_freeze_sha256"] == sha(ROOT / "PREPARATION-FREEZE.json", deadline)
            and producer["gates_binding_sha256"] == GATES_BINDING_SHA, "Producer preparation scope differs")
    prep_auth = read(descriptor(producer["preparation_authorization"], ROOT / "preparation-authorization.json", deadline))
    verify_preparation_scope(prep_auth)
    for key, name in (("plan", "PROTOCOL.json"), ("source_input_binding", "SOURCE-BINDING.json"),
                      ("preparation_freeze", "PREPARATION-FREEZE.json"), ("gates", "GATES-BINDING.json")):
        descriptor(prep_auth[key], ROOT / name, deadline)
    require(type(producer["build_confirmation_exit_code"]) is int and producer["build_confirmation_exit_code"] == 0
            and producer["private_real_target_no_symlinks"] is True
            and producer["target_initially_empty"] is True
            and producer["working_directory"] == str(ROOT.relative_to(REPO)) and producer["cargo_target_dir"] == "target"
            and producer["command"] == ["cargo", "build", "--release", "--locked", "--manifest-path", "harness/Cargo.toml"], "Fresh locked producer build not confirmed")
    require(sha(safe_file("build-confirmation.log", ROOT), deadline) == producer["build_confirmation_log_sha256"], "Producer build log differs")
    require(set(producer["files"]) == {"harness/Cargo.toml", "harness/Cargo.lock", "harness/src/main.rs"}, "Wrong producer instrumentation inventory")
    for name, digest in producer["files"].items():
        require(sha(safe_file(name, ROOT), deadline) == digest, "Producer instrumentation differs")
    require(producer["driver_path"] == "candidate/harbor-public-driver", "Wrong actual producer path")
    require(sha(safe_file(producer["driver_path"], ROOT), deadline) == producer["driver_sha256"], "Actual new producer bytes differ")
    files_below(ROOT / "target")
    require(type(producer["geometry_only_executions"]) is int and producer["geometry_only_executions"] == 2
            and type(producer["physics_initializations_or_steps"]) is int and producer["physics_initializations_or_steps"] == 0, "Unexpected preparation workload")
    gold = REPO / ".local/harbor-contact-candidates-20261114/public-safe-candidate"
    require(set(producer["preflight"]) == {"base", "height"}, "Wrong preflight cases")
    for case in ("base", "height"):
        metadata = safe_file(f"preflight-{case}/mesh-metadata.json", ROOT)
        obj = safe_file(f"preflight-{case}/rest.obj", ROOT)
        bound = producer["preflight"][case]
        require(sha(metadata, deadline) == bound["metadata_sha256"] and sha(obj, deadline) == bound["rest_obj_sha256"], "New geometry preflight bytes differ")
        old = read(gold / f"runs/{case}/rest-metadata.json")
        old.pop("masses_kg")
        require(same(read(metadata), old), "Exact original serialized geometry preflight differs")
        require(obj.read_bytes() == (gold / f"prepared/{case}/rest.obj").read_bytes(), "Original rest OBJ bytes differ")
    return {**summary, "producer_binding_sha256": sha(path, deadline), "driver_sha256": producer["driver_sha256"]}


def verify_frozen(deadline=None):
    preparation = read(safe_file("PREPARATION-FREEZE.json", ROOT))
    freeze = read(safe_file("FREEZE.json", ROOT))
    require(type(freeze["schema"]) is int and freeze["schema"] == 1
            and freeze["status"] == "frozen_preparation_no_garment_execution" and freeze["source_commit"] == SOURCE
            and freeze["new_garment_execution"] is False and freeze["files"] == preparation["files"], "Execution evaluator freeze scope differs")
    for key, name in (("plan", "PROTOCOL.json"), ("source_input_binding", "SOURCE-BINDING.json"),
                      ("producer_binding", "PRODUCER-BINDING.json"), ("preparation_freeze", "PREPARATION-FREEZE.json"),
                      ("readonly_input_binding", "READONLY-INPUT-BINDING.json"), ("gates", "GATES-BINDING.json")):
        descriptor(freeze[key], ROOT / name, deadline)
    verify_preparation(deadline)
    return freeze


def authorization(path, expected_auth_sha=None, deadline=None):
    path = safe_file(path, ROOT)
    digest = sha(path, deadline)
    require(expected_auth_sha is None or digest == expected_auth_sha, "Root authorization changed")
    auth = read(path)
    require(type(auth["schema"]) is int and auth["schema"] == 1 and auth["status"] == "authorized"
            and auth["explicit_root_authorization"] is True, "Separate explicit root execution authorization absent")
    require(auth["source_commit"] == SOURCE and auth["cases"] == ["base", "height"]
            and type(auth["attempts_per_case"]) is int and auth["attempts_per_case"] == 1, "Root authorization workload differs")
    for key, name in (("runner", "runner.py"), ("plan", "PROTOCOL.json"), ("source_input_binding", "SOURCE-BINDING.json"),
                      ("producer_binding", "PRODUCER-BINDING.json"), ("evaluator_freeze", "FREEZE.json"),
                      ("preparation_freeze", "PREPARATION-FREEZE.json"), ("gates", "GATES-BINDING.json")):
        descriptor(auth[key], ROOT / name, deadline)
    verify_frozen(deadline)
    return {"auth": auth, "auth_path": path, "authorization_sha256": digest,
            "producer": read(ROOT / "PRODUCER-BINDING.json"), "plan": read(ROOT / "PROTOCOL.json"),
            "source_commit": SOURCE, "evaluator_freeze_sha256": sha(ROOT / "FREEZE.json", deadline)}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("inspect", "verify-producer"))
    args = parser.parse_args()
    value = verify_static() if args.command == "inspect" else verify_preparation()
    print(json.dumps({"status": "verified", "new_garment_execution": False, **value}, allow_nan=False))
