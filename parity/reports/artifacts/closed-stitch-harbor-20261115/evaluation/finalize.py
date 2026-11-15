"""Write a new execution-preparation freeze; never authorizes or runs dynamics."""
import json
from pathlib import Path
import sys

sys.dont_write_bytecode = True
import bindings as b


def descriptor(name):
    path = b.ROOT / name
    return {"path": str(path.relative_to(b.REPO)), "sha256": b.sha(path)}


def main():
    summary = b.verify_preparation()
    preparation = b.read(b.ROOT / "PREPARATION-FREEZE.json")
    value = {"schema": 1, "status": "frozen_preparation_no_garment_execution", "source_commit": b.SOURCE,
             "new_garment_execution": False, "files": preparation["files"],
             "plan": descriptor("PROTOCOL.json"), "source_input_binding": descriptor("SOURCE-BINDING.json"),
             "producer_binding": descriptor("PRODUCER-BINDING.json"), "preparation_freeze": descriptor("PREPARATION-FREEZE.json"),
             "readonly_input_binding": descriptor("READONLY-INPUT-BINDING.json"), "gates": descriptor("GATES-BINDING.json"),
             "verified_preparation": summary, "scope": "Preparation freeze only; separate explicit root execution authorization required"}
    path = b.ROOT / "FREEZE.json"
    with path.open("x") as stream:
        stream.write(json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n")
    b.verify_frozen()
    print(json.dumps({"status": value["status"], "path": str(path.relative_to(b.REPO)), "sha256": b.sha(path), "new_garment_execution": False}))


if __name__ == "__main__":
    main()
