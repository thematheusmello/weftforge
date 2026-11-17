"""After root supplies reviewed candidate metadata, bind source only; never build/init."""
import json
import argparse
from source_inputs import create_preflight, verify_preflight

if __name__=="__main__":
    parser=argparse.ArgumentParser();parser.add_argument("--verify-only",action="store_true");args=parser.parse_args()
    result=verify_preflight() if args.verify_only else create_preflight()
    print(json.dumps({"status":result["status"],"source_commit":result["source_commit"],"build_binding_sha256":result["build_binding_sha256"],"Cloth_initializations":0,"Cloth_steps":0}))
