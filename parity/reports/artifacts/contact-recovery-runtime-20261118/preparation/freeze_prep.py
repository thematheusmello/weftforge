"""Final source/binary seal only after distinct reviewed-candidate build; no constructor."""
import json
from common import ROOT, REPO, r
import common as b
from source_inputs import verify_preflight, INPUT, GOLD, READONLY, code_paths


def main():
    preflight=verify_preflight()
    r.require(not (ROOT/"FREEZE.json").exists() and not (ROOT/"PROTOCOL.json").exists(),"Do not overwrite a final preparation")
    r.require(not any((ROOT/name).exists() for name in r.OUTPUT_DIRECTORIES),"A numerical case already started")
    r.require(not (ROOT/"ROOT-GO.json").exists(),"Preparation must not create execution GO")
    build=b.read(r.safe_file("checks/BUILD-RECEIPT.json",ROOT));stage=build["stage"]
    r.require(build["source_commit"]==b.SOURCE and build["source_preflight_sha256"]==r.sha(ROOT/"SOURCE-PREFLIGHT.json") and build["build_binding_sha256"]==preflight["build_binding_sha256"],"Built source/compiler binding differs")
    r.require(build["reviewed_source_prerequisites_passed"] is True,"Build lacks reviewed source-prerequisite approval")
    b.c.descriptor(build["source_prerequisite_receipt"],REPO)
    r.require(stage["exit_code"]==0 and stage["stop_reason"] is None and stage["end_monotonic_seconds"]<stage["terminal_phase_checks"]["deadline"],"Release build failed its phase caps")
    binary=build["binary"];r.require(r.sha(r.safe_file(binary["path"],REPO))==binary["sha256"],"Own release binary changed")
    reused=b.read(ROOT/"REUSED-SOURCES.json")
    for name,digest in reused["runtime_files"].items():
        if name.startswith(("consumer/","producer/")):r.require(r.sha(ROOT/name)==digest,"Approved independent evaluator/producer source changed")
    protocol=b.read(ROOT/"PROTOCOL-DRAFT.json")
    protocol.update(status="prepared_not_authorized",source_commit=b.SOURCE,source_pending=False,cwd=str(ROOT),python=preflight["python"],build_binding_sha256=preflight["build_binding_sha256"],producer_binary=binary)
    python=preflight["python"]["binary_path"];actual_binary=str(REPO/binary["path"])
    protocol["argv"]={"controller":[python,"-B",str(ROOT/"controller.py"),"--go","ROOT-GO.json","--go-sha256","<EXACT_NEW_ROOT_GO_SHA256>"],"init_only":[actual_binary,"prepare","inputs/base.weftforge","preflight-base"],"original_metadata_gate":[python,"-B",str(ROOT/"worker.py"),"metadata","--deadline","<ONE_ABSOLUTE_DEADLINE>"],"producer":[actual_binary,"run","inputs/base.weftforge","runs/base","expected/approved-metadata.json"],"analysis":[python,"-B",str(ROOT/"worker.py"),"analyze","--deadline","<ONE_ABSOLUTE_DEADLINE>"],"finalization":[python,"-B",str(ROOT/"worker.py"),"finalize","--deadline","<ONE_ABSOLUTE_DEADLINE>"]}
    b.persist(ROOT/"PROTOCOL.json",protocol)
    repo_files=dict(READONLY)
    for key in ["source_manifest","source_review_evidence","green_checks_evidence"]:
        path=b._candidate[key];repo_files[path]=r.sha(r.safe_file(path,REPO))
    frozen={"schema":2,"experiment":b.EXPERIMENT,"source_commit":b.SOURCE,"status":"prepared_no_GO_no_Harbor_execution","source_path":b._candidate["source_manifest"],"repo_files":repo_files,"files":{str(p.relative_to(ROOT)):r.sha(p) for p in r.files_below(ROOT) if p.name!="FREEZE.json"},"code_paths":code_paths(),"python":preflight["python"],"producer_binary":binary,"input_path":INPUT,"input_sha256":READONLY[INPUT],"rest_sha256":READONLY[GOLD],"build_binding_sha256":preflight["build_binding_sha256"],"fresh_initializations":2,"init_only_steps":0,"actual_dynamics_runs":1}
    digest=b.persist(ROOT/"FREEZE.json",frozen);b.verify_bound()
    print(json.dumps({"status":"prepared_not_authorized","freeze_sha256":digest,"build_binding_sha256":preflight["build_binding_sha256"],"producer_sha256":binary["sha256"],"source_commit":b.SOURCE,"Cloth_initializations":0,"Cloth_steps":0}))
    return 0


if __name__=="__main__":raise SystemExit(main())
