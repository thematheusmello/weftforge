"""Read-only closed candidate and incomplete-prefix scope checks."""
from pathlib import Path
import hashlib,json
r=Path(__file__).resolve().parent
m=json.loads((r/"MANIFEST.json").read_text())
actual={str(p.relative_to(r)) for p in r.glob("**/*") if p.is_file() and p.name!="MANIFEST.json"}
assert actual==set(m["files"]),"Candidate file closure differs"
for name,expected in m["files"].items():
 p=r/name
 assert not p.is_symlink() and p.stat().st_size==expected["bytes"]
 assert hashlib.sha256(p.read_bytes()).hexdigest()==expected["sha256"],name
 assert p.suffix.lower()!=".png"
 if p.suffix.lower()!=".jpg":
  text=p.read_text()
  assert all(token not in text for token in ("/"+"Users/","/"+"home/","/"+"private/var/")),name
for name,count in (("base",164),("height",159)):
 a=json.loads((r/f"partial-review/results/{name}-prefix.json").read_text())
 assert a["status"]=="incomplete_prefix_only" and len(a["rows"])==count
 assert a["arithmetic_consistency"]=="verified_on_all_available_states"
 assert a["overall_quality"]=="not_qualified_incomplete_prefix_only"
 assert not (r/f"runs/{name}/completed-case.json").exists() and not (r/f"runs/{name}/stage-180.obj").exists()
scope=json.loads((r/"partial-review/results/scope.json").read_text())
assert scope["status"]=="incomplete_prefix_only"
native=json.loads((r/"partial-review/results/native-available.json").read_text())
assert [row["frame"] for row in native["accepted_checkpoints"]]==[0,60] and native["native_A_180"]=="missing"
print(json.dumps({"closed_files":len(actual),"bytes":sum(v["bytes"] for v in m["files"].values()),"status":"incomplete_prefix_only","manifest_integrity":"verified","physical_runs_added":0}))
