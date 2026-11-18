"""Fixed reviewed-candidate source/compiler binding; never constructs a model."""
import hashlib
import os
from pathlib import Path
from datetime import datetime, timezone

import common as b
from common import ROOT, REPO, r
from compiler_inputs import local_inventory, dependency_inventory, cargo_config_inventory, ambient_build_env

INPUT=".local/harbor-contact-recovery-preparation-20261118/original-fixtures/base.weftforge"
GOLD=".local/harbor-contact-recovery-preparation-20261118/original-fixtures/rest-metadata.json"
READONLY={INPUT:"fc494974955d440aacbfde49c332a50218bd3a9354d06f4fbbf0596e5da8eda7",GOLD:"d96697949c686fbf15f19b99b6d7551b35d3449b17157009aceb75649fbaeeb9"}


def code_paths():
    return sorted(str(p.relative_to(ROOT)) for directory,pattern in [(ROOT,"*.py"),(ROOT/"consumer","*.py"),(ROOT/"producer/src","*.rs")] for p in directory.glob(pattern))


def clean_child_env(binding):
    for key in ["RUSTC_WRAPPER","RUSTC_WORKSPACE_WRAPPER","CARGO_BUILD_RUSTC_WRAPPER","CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER"]:
        r.require(os.environ.get(key) in (None,""),"Unexpected compiler wrapper: "+key)
    for key in ["RUSTC","RUSTDOC","RUSTUP_TOOLCHAIN"]:
        r.require(os.environ.get(key) in (None,"",binding["child_env"][key]),"Unexpected inherited "+key)
    r.require(os.environ.get("PATH","")==binding["parent_path"],"Parent PATH changed after compiler binding")
    r.require(ambient_build_env()==binding["ambient_build_env"],"Cargo build/profile/target environment changed")
    r.require(all(os.environ.get(key,"")==binding["child_env"][key] for key in ["RUSTFLAGS","CARGO_ENCODED_RUSTFLAGS","RUSTDOCFLAGS"]),"Rust/compiler documentation flags changed")
    r.require(str(Path(os.environ.get("CARGO_HOME") or Path.home()/".cargo").resolve())==binding["cargo_home"],"Cargo home changed")
    env=dict(os.environ)
    for key in ["CARGO_BUILD_RUSTC_WRAPPER","CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER"]:env.pop(key,None)
    env.update(binding["child_env"])
    return env


def create_preflight():
    b.require_source()
    r.no_links(REPO/b.OWN_TARGET,REPO)
    r.require(not (ROOT/"SOURCE-PREFLIGHT.json").exists(),"Do not replace a bound source preparation")
    for name,digest in READONLY.items():r.require(r.sha(r.safe_file(name,REPO))==digest,"Original input/rest changed")
    for name,digest in b.read(ROOT/"REUSED-SOURCES.json")["runtime_files"].items():
        if name.startswith(("consumer/","producer/")):r.require(r.sha(ROOT/name)==digest,"Approved producer/evaluator source changed")
    toolchain=Path(b._candidate["rust_toolchain_bin"])
    r.require(toolchain.is_absolute() and toolchain.name=="bin" and toolchain.parent.name=="1.94.1-aarch64-apple-darwin","Wrong fixed toolchain directory")
    compiler=toolchain/"rustc";cargo=toolchain/"cargo";rustdoc=toolchain/"rustdoc"
    for executable in [compiler,cargo,rustdoc]:
        r.require(executable.is_file() and not executable.is_symlink(),"Actual compiler tool missing or symlinked: "+executable.name)
    parent_path=b._candidate["approved_parent_path"]
    r.require(os.environ.get("PATH","")==parent_path,"Use the root-approved stable parent PATH before binding")
    child_path=parent_path
    cargo_home=str(Path(os.environ.get("CARGO_HOME") or Path.home()/".cargo").resolve())
    binding={"rustc":{"path":str(compiler),"sha256":r.sha(compiler)},"cargo":{"path":str(cargo),"sha256":r.sha(cargo)},"rustdoc":{"path":str(rustdoc),"sha256":r.sha(rustdoc)},"argv":[str(cargo),"build","--release","--locked","--offline","--manifest-path",str(ROOT/"producer/Cargo.toml")],"cargo_home":cargo_home,"parent_path":parent_path,"ambient_build_env":ambient_build_env(),"child_env":{"RUSTC":str(compiler),"RUSTDOC":str(rustdoc),"RUSTUP_TOOLCHAIN":"1.94.1","PATH":child_path,"RUSTC_WRAPPER":"","RUSTC_WORKSPACE_WRAPPER":"","RUSTFLAGS":os.environ.get("RUSTFLAGS",""),"CARGO_ENCODED_RUSTFLAGS":os.environ.get("CARGO_ENCODED_RUSTFLAGS",""),"RUSTDOCFLAGS":os.environ.get("RUSTDOCFLAGS",""),"CARGO_BUILD_JOBS":"1","CARGO_TARGET_DIR":str(REPO/b.OWN_TARGET),"CARGO_HOME":cargo_home,"PYTHONOPTIMIZE":"0","PYTHONDONTWRITEBYTECODE":"1"},"scope_limit":"Actual Cargo/rustc/rustdoc executables, full child PATH/toolchain/known flags, local/config and locked crate source inputs are bound. Linker/macOS SDK/sysroot shared-library closure is not certified."}
    clean_child_env(binding)
    binding["local_inputs"]=local_inventory(REPO)
    binding["dependency_sources"]=dependency_inventory(REPO,cargo_home)
    binding["cargo_config_inputs"]=cargo_config_inventory(REPO,cargo_home)
    digest=hashlib.sha256(r.encoded(binding)).hexdigest()
    paths=code_paths()+["producer/Cargo.toml","producer/Cargo.lock","CANDIDATE.json","REUSED-SOURCES.json","PROTOCOL-DRAFT.json","SOURCE.json","DRAFT-ORIGIN.json","PRESERVATION.json","DELTA.json"]
    paths += [str(p.relative_to(ROOT)) for p in sorted((ROOT/"evidence").glob("*.json"))]
    preflight={"status":"reviewed_source_bound_no_build","bound_at_utc":datetime.now(timezone.utc).isoformat(),"source_commit":b.SOURCE,"candidate_binding_sha256":r.sha(ROOT/"CANDIDATE.json"),"python":b.environment(),"code_paths":code_paths(),"files":{name:r.sha(ROOT/name) for name in paths},"original_input_rest":READONLY,"build_binding":binding,"build_binding_sha256":digest,"source_review_sha256":b._candidate["source_review_sha256"],"green_checks_sha256":b._candidate["green_checks_sha256"],"workspace_gate_status_at_binding":b._candidate["workspace_gates_status_at_binding"],"build_authorized":False,"own_target_preexisting":(REPO/b.OWN_TARGET).exists(),"same_checkout_target_reuse_allowed":True,"compiled_target_cross_copy":False,"Cloth_initializations":0,"Cloth_steps":0,"Cargo_invocations":0}
    b.validate_environment(preflight["python"])
    b.persist(ROOT/"SOURCE-PREFLIGHT.json",preflight)
    return preflight


def verify_preflight(deadline=None):
    b.require_source(deadline)
    value=b.read(r.safe_file("SOURCE-PREFLIGHT.json",ROOT))
    r.require(value["source_commit"]==b.SOURCE and value["candidate_binding_sha256"]==r.sha(ROOT/"CANDIDATE.json",deadline),"Candidate binding changed")
    r.require(code_paths()==value["code_paths"],"Program/import source roster changed")
    for name,digest in value["files"].items():r.require(r.sha(r.safe_file(name,ROOT),deadline)==digest,"Program source changed: "+name)
    for name,digest in READONLY.items():r.require(r.sha(r.safe_file(name,REPO),deadline)==digest,"Original input/rest changed")
    binding=value["build_binding"]
    r.require(hashlib.sha256(r.encoded(binding)).hexdigest()==value["build_binding_sha256"],"Build binding changed")
    for name in ["rustc","cargo","rustdoc"]:r.require(r.sha(binding[name]["path"],deadline)==binding[name]["sha256"],"Compiler executable changed")
    r.require(local_inventory(REPO)==binding["local_inputs"],"Added/deleted/changed local compiler source, including build.rs")
    r.require(dependency_inventory(REPO,binding["cargo_home"])==binding["dependency_sources"],"Immutable dependency sources changed")
    r.require(cargo_config_inventory(REPO,binding["cargo_home"])==binding["cargo_config_inputs"],"Cargo configuration/absence changed")
    clean_child_env(binding);b.validate_environment(value["python"],deadline);r.deadline_check(deadline)
    return value
