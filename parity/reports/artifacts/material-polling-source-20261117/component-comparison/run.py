import time
STARTED = time.monotonic()
import hashlib
import json
import os
from pathlib import Path
import resource
import subprocess
import sys
import resources as r

ROOT = Path(__file__).resolve().parent
WT = ROOT.parent.parent
CAP_SECONDS = 120.
CAP_BYTES = 8388608
RSS_BYTES = 1073741824
DEADLINE = STARTED + CAP_SECONDS


def read(path):
    return json.loads(Path(path).read_text())


def persist(path, value):
    r.write_new(path, r.encoded(value))
    with Path(path).open("rb") as stream:
        os.fsync(stream.fileno())


def output_bytes():
    return sum(p.stat().st_size for p in r.files_below(ROOT/"runtime"))


def verify(freeze):
    for name, digest in freeze["files"].items():
        r.require(r.sha(WT/name, DEADLINE) == digest, "Frozen source/input changed: "+name)
    for name, mode in freeze["modes"].items():
        r.require((WT/name).stat().st_mode & 0o777 == mode, "Frozen input mode changed: "+name)
    r.require(subprocess.check_output(["/usr/bin/git","rev-parse","HEAD"],cwd=WT,text=True).strip() == freeze["candidate_commit"], "Candidate commit changed")
    actual = sorted(str(q.relative_to(WT)) for folder in [ROOT,WT/"crates/weftforge-sim"] for q in folder.rglob("*.rs"))
    r.require(actual == freeze["compiler_source_inventory"] and not (ROOT/"producer/build.rs").exists() and not (WT/"crates/weftforge-sim/build.rs").exists(), "Compiler input source inventory changed")
    for item in freeze["tools"].values():
        r.require(r.sha(item["path"], DEADLINE) == item["sha256"], "Tool executable changed")
    r.deadline_check(DEADLINE)


def main():
    r.require(sys.flags.optimize == 0 and sys.dont_write_bytecode and os.environ.get("PYTHONOPTIMIZE") in (None,"0"), "Use Python optimization0 and -B")
    freeze = read(ROOT/"FREEZE.json")
    r.require(not (ROOT/"runtime").exists(), "One fixed command only; no retry")
    (ROOT/"runtime").mkdir()
    stages, errors = {}, []
    persist(ROOT/"runtime/STARTED.json", {"started":STARTED,"deadline":DEADLINE,"pid":os.getpid(),
        "argv":sys.argv,"source_freeze_sha256":r.sha(ROOT/"FREEZE.json"),
        "caps":{"seconds":120,"aggregate_sampled_group_plus_supervisor_RSS":RSS_BYTES,"retained_output_bytes":CAP_BYTES}})
    try:
        verify(freeze)
        for key in ["RUSTC_WRAPPER","RUSTC_WORKSPACE_WRAPPER","CARGO_BUILD_RUSTC_WRAPPER","CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER"]:
            r.require(os.environ.get(key) in (None,""), "Unexpected wrapper "+key)
        env = dict(os.environ)
        for key in ["CARGO_BUILD_RUSTC_WRAPPER","CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER"]:
            env.pop(key,None)
        env.update(freeze["child_env"])
        for name, argv in [("build",freeze["build_argv"]),("kernel",freeze["kernel_argv"])]:
            stages[name] = r.monitor(argv,WT,ROOT/f"runtime/{name}.stdout",ROOT/f"runtime/{name}.stderr",
                DEADLINE,output_bytes,rss_cap=RSS_BYTES,output_cap=CAP_BYTES,reserve=131072,child_env=env,
                on_start=lambda value,n=name:persist(ROOT/f"runtime/{n}-STARTED.json",value))
            persist(ROOT/f"runtime/{name}-RESULT.json",stages[name])
            r.require(stages[name]["stop_reason"] is None and stages[name]["exit_code"] == 0, name+" did not complete under shared caps")
        rows = [json.loads(line) for line in (ROOT/"runtime/kernel.stdout").read_text().splitlines()]
        r.require(rows[0]["kind"]=="fixture" and rows[0]["vertices"]==651 and rows[0]["triangles"]==1200, "Fixture count")
        r.require(rows[0]["cancellation_predicate"] == "|| started.elapsed().as_secs_f64() >= 420." and rows[0]["per_callback_counter"] is False, "Clock-only predicate binding")
        repeats=rows[1:]
        r.require(len(repeats)==6 and [x["index"] for x in repeats]==list(range(6)), "Fixed repeats changed")
        r.require([x["warmup"] for x in repeats]==[True,False,False,False,False,False], "Fixed warmup changed")
        r.require(all(x["material_calls"]==32 and x["all32_position_and_p_boundaries_bit_identical"] is True for x in repeats), "Control/observer mismatch")
        verify(freeze)
    except BaseException as error:
        errors.append(type(error).__name__+": "+str(error))
    self_rss=int(resource.getrusage(resource.RUSAGE_SELF).ru_maxrss)
    if self_rss>RSS_BYTES:
        errors.append("Supervisor exact RSS cap")
    result={"status":"COMPLETE_OLD_NEW_COMPONENT_COMPARISON" if not errors else "FAILED_OLD_NEW_COMPONENT_COMPARISON",
        "errors":errors,"stages":stages,"supervisor_exact_peak_RSS_bytes":self_rss,
        "elapsed_before_result_persistence_seconds":time.monotonic()-STARTED,
        "retained_output_bytes_before_result":output_bytes(),"source_freeze_sha256":r.sha(ROOT/"FREEZE.json"),
        "no_garment_or_quality_or_fullcase_performance_acceptance":True,"retry_or_adaptive_change":False}
    persist(ROOT/"runtime/RESULT.json",result)
    terminal={"result_sha256":r.sha(ROOT/"runtime/RESULT.json"),"elapsed_after_result_persistence_seconds":time.monotonic()-STARTED,
        "stage_PIDs":[x["pid"] for x in stages.values()],"all_direct_children_reaped":True,
        "output_bytes_before_terminal":output_bytes(),"scope":"Same-build52099-vs9d4b material-only comparison; no garment acceptance"}
    persist(ROOT/"runtime/TERMINAL.json",terminal)
    elapsed=time.monotonic()-STARTED;volume=output_bytes()
    accepted=not errors and elapsed<120. and volume+65536<8388608 and self_rss<=RSS_BYTES
    print(json.dumps({"status":result["status"],"command_complete_under_caps":accepted,
        "elapsed_after_all_persistence_seconds":elapsed,"output_bytes":volume,
        "terminal_sha256":r.sha(ROOT/"runtime/TERMINAL.json"),"no_fullcase_or_physics_acceptance":True}))
    return 0 if accepted else 1


if __name__=="__main__":
    raise SystemExit(main())
