"""Pure binding/AST mutations; never launches an engine, evaluator or producer."""
import argparse
import ast
import copy
import json
from pathlib import Path
import sys
import tempfile

sys.dont_write_bytecode = True
import bindings as b


def refused(call):
    try:
        call()
    except (ValueError, KeyError, TypeError, FileNotFoundError, FileExistsError):
        return
    raise AssertionError("Expected closed refusal")


def function_map(path):
    return {n.name: ast.dump(n, include_attributes=False) for n in ast.parse(path.read_text()).body if isinstance(n, ast.FunctionDef)}


def nonbinding_ast(path, functions, assignments):
    tree = ast.parse(path.read_text())
    tree.body = [n for n in tree.body if not (
        isinstance(n, ast.FunctionDef) and n.name in functions
        or isinstance(n, ast.Assign) and all(isinstance(t, ast.Name) and t.id in assignments for t in n.targets)
        or isinstance(n, ast.Import) and len(n.names) == 1 and n.names[0].name == "bindings")]
    return ast.dump(tree, include_attributes=False)


def reuse_proof():
    protocol = b.read(b.ROOT / "PROTOCOL.json")
    old = b.descriptor(protocol["original_evaluator_freeze"]).parent
    freeze = b.read(old / "FREEZE.json")
    modules = ["metrics.py", "geometry_math.py", "static_geometry.py", "decimal_oracle.py", "capsule_geometry.py"]
    for name in modules:
        assert b.sha(old / name) == freeze["files"][name] == b.sha(b.ROOT / name)
    before, after = function_map(old / "evaluator.py"), function_map(b.ROOT / "evaluator.py")
    adapted = {"verify_preparation", "verify_frozen", "verify_readonly_inputs", "authorization"}
    assert before.keys() == after.keys() and {n for n in before if before[n] != after[n]} == adapted
    evaluator_assignments = {"EXPERIMENT", "SOURCE", "PREVIOUS_PLAN_SHA", "PLAN_SHA", "PRODUCER_BINDING_SHA"}
    assert nonbinding_ast(old / "evaluator.py", adapted, evaluator_assignments) == nonbinding_ast(b.ROOT / "evaluator.py", adapted, evaluator_assignments)
    previous_runner = b.descriptor(protocol["original_runner"])
    rb, ra = function_map(previous_runner), function_map(b.ROOT / "runner.py")
    assert rb.keys() == ra.keys() and {n for n in rb if rb[n] != ra[n]} == {"verify_bindings"}
    runner_assignments = {"EVALUATOR", "SOURCE", "PLAN_SHA", "INPUT_BINDING_SHA", "PRODUCER_BINDING_SHA", "DRIVER_SHA"}
    assert nonbinding_ast(previous_runner, {"verify_bindings"}, runner_assignments) == nonbinding_ast(b.ROOT / "runner.py", {"verify_bindings"}, runner_assignments)
    return {"unchanged_whole_numerical_modules": {name: b.sha(b.ROOT / name) for name in modules},
            "original_evaluator_sha256": b.sha(old / "evaluator.py"), "adapted_evaluator_sha256": b.sha(b.ROOT / "evaluator.py"),
            "evaluator_binding_function_allowlist": sorted(adapted), "unchanged_evaluator_functions": sorted(set(before)-adapted),
            "original_runner_sha256": b.sha(previous_runner), "adapted_runner_sha256": b.sha(b.ROOT / "runner.py"),
            "runner_binding_function_allowlist": ["verify_bindings"], "unchanged_runner_functions": sorted(set(rb)-{"verify_bindings"}),
            "all_other_module_AST_nodes_and_constants_unchanged": True,
            "runner_selfcheck_source_byte_identical": b.sha(b.ROOT / "runner-selfchecks.py") == b.sha(previous_runner.parent / "runner-selfchecks.py"),
            "numeric_functions_settings_thresholds_and_aggregation_unchanged": True}


def main(output):
    static = b.verify_static(require_freeze=False)
    gates = b.verify_gates()
    proof = reuse_proof()
    assert proof["runner_selfcheck_source_byte_identical"]
    checks = ["Exact source45/input2/prior202 hashes", "All five completed gates and actual native/CLI bytes", "Five numerical modules byte-identical; all numerical evaluator and nonbinding runner functions AST-identical"]
    source, protocol = b.pinned_source(), b.read(b.ROOT / "PROTOCOL.json")
    read_original, sha_original = b.read, b.sha
    try:
        for field, value in (("schema", True), ("source_commit", None), ("source_commit", "0770204a54abdf5f929b6626b01d7fcfb0e13ca4"),
                             ("source_commit", "Z"*40), ("source_hashes", {})):
            changed = copy.deepcopy(source)
            changed[field] = value
            b.read = lambda path, changed=changed: changed if Path(path) == b.ROOT / "SOURCE-BINDING.json" else read_original(path)
            b.sha = lambda path, deadline=None: b.SOURCE_BINDING_SHA if Path(path) == b.ROOT / "SOURCE-BINDING.json" else sha_original(path, deadline)
            refused(b.pinned_source)
    finally:
        b.read, b.sha = read_original, sha_original
    pinned_original = b.pinned_source
    try:
        for mutation in ("missing_path", "extra_path", "outside_three_file_delta"):
            changed = copy.deepcopy(source)
            if mutation == "missing_path":
                changed["source_hashes"].pop(next(iter(changed["source_hashes"])))
            elif mutation == "extra_path":
                changed["source_hashes"]["../extra.rs"] = "0"*64
            else:
                unchanged = next(name for name in protocol["source_file_paths"] if name not in protocol["allowed_changed_runtime_paths"])
                changed["source_hashes"][unchanged] = "0"*64
            b.pinned_source = lambda changed=changed: changed
            refused(lambda: b.verify_static(require_freeze=False))
    finally:
        b.pinned_source = pinned_original
    checks.append("Source finalization, old-source reuse, path inventory and undeclared fourth source change refused")
    prep_auth = {"schema": 1, "status": "authorized_preparation_only", "explicit_root_preparation_authorization": True,
                 "source_commit": b.SOURCE, "cases": ["base", "height"], "locked_driver_build_authorized": True,
                 "geometry_only_executions": 2, "physics_initializations_or_steps": 0, "dynamics_authorized": False}
    b.verify_preparation_scope(prep_auth)
    for field, value in (("schema", True), ("status", "authorized"), ("explicit_root_preparation_authorization", False),
                         ("geometry_only_executions", True), ("geometry_only_executions", 3),
                         ("physics_initializations_or_steps", False), ("physics_initializations_or_steps", 1), ("dynamics_authorized", True)):
        changed = copy.deepcopy(prep_auth)
        changed[field] = value
        refused(lambda changed=changed: b.verify_preparation_scope(changed))
    checks.append("Separate build/two-mesh preparation authority cannot grant dynamics or stand in for execution authorization")
    with tempfile.TemporaryDirectory(prefix="binding-selfcheck-", dir=b.ROOT) as directory:
        directory = Path(directory)
        for i, text in enumerate(('{"x":1,"x":2}', '{"x":{"y":0,"y":1}}', '{"x":NaN}', '{"x":1e999}', '{"x":'+('9'*1000)+'}')):
            p = directory / f"invalid-{i}.json"
            p.write_text(text)
            refused(lambda p=p: b.read(p))
        asset = directory / "asset"
        asset.write_bytes(b"preserve")
        link = directory / "link"
        link.symlink_to(asset)
        refused(lambda: b.safe_file(link, directory))
        refused(lambda: b.safe_file("../asset", directory))
        refused(lambda: b.descriptor({"path": str(asset), "sha256": b.sha(asset)}))
        refused(lambda: b.descriptor({"path": str(asset.relative_to(b.REPO)), "sha256": "0"*64}))
        valid = {"schema": 1, "status": "authorized", "explicit_root_authorization": True, "source_commit": b.SOURCE,
                 "cases": ["base", "height"], "attempts_per_case": 1}
        for field, value in (("schema", True), ("status", "prepared"), ("explicit_root_authorization", False),
                             ("attempts_per_case", True), ("attempts_per_case", 2), ("source_commit", "stale"),
                             ("cases", ["height", "base"]), ("cases", ["base", "height", "third"])):
            changed = copy.deepcopy(valid)
            changed[field] = value
            path = directory / "not-authorized.json"
            path.write_text(json.dumps(changed))
            refused(lambda: b.authorization(path))
    checks.append("Duplicate/nonfinite/overflow external JSON, symlink/traversal/bad descriptors, bool schema/attempt and invalid root authorizations refused")
    if not (b.ROOT / "PRODUCER-BINDING.json").exists():
        refused(b.verify_preparation)
        checks.append("Missing fresh producer cannot satisfy execution preparation")
    else:
        b.verify_preparation()
        checks.append("Existing producer preparation validates without executing it")
    if not (b.ROOT / "FREEZE.json").exists():
        refused(b.verify_frozen)
        checks.append("Missing final freeze cannot satisfy execution authorization")
    else:
        b.verify_frozen()
        checks.append("Existing final freeze validates without executing cases")
    value = {"schema": 1, "status": "passed", "new_garment_execution": False, "new_product_or_engine_execution": False,
             "new_physics_initializations_or_steps": 0, "checks": checks, "static_binding": static,
             "gated_native_sha256": gates["native"]["sha256"], "gated_cli_sha256": gates["cli"]["sha256"], "reuse_proof": proof}
    path = Path(output).absolute()
    b.require(path.resolve().is_relative_to(b.ROOT), "Selfcheck output outside own preparation")
    with path.open("x") as stream:
        stream.write(json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n")
    print(json.dumps({"status": "passed", "checks": len(checks), "new_garment_execution": False, "output": str(path.relative_to(b.REPO))}))


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, required=True)
    main(parser.parse_args().output)
