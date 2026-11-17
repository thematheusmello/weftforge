"""Distinct root-GO release build only; no model constructor or numerical producer."""
import argparse
import json
import time
from common import ROOT, REPO, r
import common as b
from source_inputs import verify_preflight, clean_child_env


def main():
    parser=argparse.ArgumentParser();parser.add_argument("--go",required=True);parser.add_argument("--go-sha256",required=True);args=parser.parse_args()
    preflight=verify_preflight()
    path=r.safe_file(args.go,ROOT);r.require(r.sha(path)==args.go_sha256,"Build GO hash changed")
    authorization=b.read(path)
    workspace_receipt=authorization["workspace_gate_receipt"]
    b.c.descriptor(workspace_receipt,REPO)
    expected={"status":"authorized_build_only","explicit_root_review_and_go":True,"experiment":b.EXPERIMENT,"source_commit":b.SOURCE,"source_preflight_sha256":r.sha(ROOT/"SOURCE-PREFLIGHT.json"),"build_binding_sha256":preflight["build_binding_sha256"],"full_workspace_gates_passed":True,"workspace_gate_receipt":workspace_receipt,"Cloth_initializations":0,"Cloth_steps":0}
    r.require(b.c.same(authorization,expected),"Separate exact build-only root GO and closed workspace gates missing")
    target=r.no_links(REPO/b.OWN_TARGET,REPO)
    out=ROOT/"checks/build";out.mkdir();started=time.monotonic();deadline=started+120.
    result=r.monitor(preflight["build_binding"]["argv"],REPO,out/"stdout",out/"stderr",deadline,lambda:r.directory_bytes([out],deadline=deadline),rss_cap=1073741824,output_cap=8388608,reserve=2*r.RECEIPT_RESERVE,child_env=clean_child_env(preflight["build_binding"]),on_start=lambda handle:b.persist(out/"started.json",handle,deadline))
    receipt={"source_commit":b.SOURCE,"source_preflight_sha256":r.sha(ROOT/"SOURCE-PREFLIGHT.json"),"build_binding_sha256":preflight["build_binding_sha256"],"build_binding":preflight["build_binding"],"full_workspace_gates_passed":True,"workspace_gate_receipt":workspace_receipt,"build_GO_sha256":args.go_sha256,"stage":result,"Cloth_initializations":0,"Cloth_steps":0,"binary_copied":False,"same_checkout_existing_target_allowed":True}
    if result["stop_reason"] is None and result["exit_code"]==0:
        verify_preflight();binary=target/"release/harbor-coupled-producer"
        receipt["binary"]={"path":str(binary.relative_to(REPO)),"sha256":r.sha(binary),"size_bytes":binary.stat().st_size}
    b.persist(ROOT/"checks/BUILD-RECEIPT.json",receipt)
    r.require(result["stop_reason"] is None and result["exit_code"]==0,"Build failed; receipt retained, no continuation")
    print(json.dumps({"status":"built_not_initialized","binary":receipt["binary"],"build_binding_sha256":preflight["build_binding_sha256"]}))
    return 0


if __name__=="__main__":raise SystemExit(main())
