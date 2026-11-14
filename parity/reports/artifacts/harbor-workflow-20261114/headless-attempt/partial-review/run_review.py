"""One bounded read-only POST-FAILURE qualification, separate from ended physical caps."""
from pathlib import Path
import datetime
import hashlib
import json
import os
import signal
import subprocess
import time

ROOT=Path(__file__).resolve().parent
ATTEMPT=ROOT.parent
NATIVE=ATTEMPT.parent/"empty-shirt-authoring-20261114/native-physics/base"
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()


def save(path,value):
    temporary=path.with_suffix(".tmp")
    temporary.write_text(json.dumps(value,indent=2)+"\n")
    temporary.replace(path)


registration=json.loads((ROOT/"registration.json").read_text())
for name,expected in registration["reader_files"].items():
    assert sha(ROOT/name)==expected,name
for name,expected in registration["attempt_files"].items():
    assert sha(ATTEMPT/name)==expected,name
for name,expected in registration["native_files"].items():
    assert sha(NATIVE/name)==expected,name
source=json.loads((ATTEMPT/"source-manifest.json").read_text())
assert all(sha(ATTEMPT/"source"/name)==expected for name,expected in source["files"].items())
assert subprocess.check_output(["git","status","--porcelain"],cwd=ATTEMPT/"source")==b""
ledger=ROOT/"checks/review-job.json"
assert not ledger.exists() and not (ROOT/"results").exists()
job={"kind":"POST-FAILURE read-only prefix arithmetic qualification, no scene or solver step","status":"ready","wrapper_pid":os.getpid(),"registered_before_data_calculation":True,"registration_sha256":sha(ROOT/"registration.json"),"started_utc":datetime.datetime.now(datetime.timezone.utc).isoformat(),"wall_cap_seconds":180,"additional_output_cap_bytes":67108864,"children":[]}
save(ledger,job)
print(json.dumps(job),flush=True)
started=time.monotonic()
try:
    argv=["python3",str(ROOT/"qualify_prefix.py"),str(ATTEMPT),str(ROOT/"results"),str(NATIVE)]
    with (ROOT/"checks/review.stdout").open("xb") as out,(ROOT/"checks/review.stderr").open("xb") as err:
        child=subprocess.Popen(argv,cwd=ROOT,env=dict(os.environ,PYTHONDONTWRITEBYTECODE="1"),stdout=out,stderr=err,start_new_session=True)
        handle={"pid":child.pid,"process_group":child.pid,"argv":argv,"status":"running"}
        job["children"].append(handle)
        job["status"]="running"
        save(ledger,job)
        print(json.dumps(handle),flush=True)
        while child.poll() is None:
            size=sum(p.stat().st_size for p in (ROOT/"results").glob("**/*") if p.is_file())
            if time.monotonic()-started>=180 or size>=67108864:
                os.killpg(child.pid,signal.SIGKILL)
                child.wait()
                handle.update(status="stopped_at_post_failure_analysis_cap",exit_code=child.returncode)
                raise RuntimeError("Fixed read-only review cap reached; no extension")
            time.sleep(.05)
        handle.update(status="completed",exit_code=child.returncode)
        if child.returncode:
            raise RuntimeError(f"Post-failure reader exit {child.returncode}; retain original outputs")
    job["status"]="completed_incomplete_prefix_only"
except Exception as exc:
    job.update(status="failed",first_failure=f"{type(exc).__name__}: {exc}")
finally:
    job.update(completed_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),wall_seconds=time.monotonic()-started)
    job["reader_files_after_sha256"]={name:sha(ROOT/name) for name in registration["reader_files"]}
    job["attempt_files_after_sha256"]={name:sha(ATTEMPT/name) for name in registration["attempt_files"]}
    job["native_files_after_sha256"]={name:sha(NATIVE/name) for name in registration["native_files"]}
    assert job["reader_files_after_sha256"]==registration["reader_files"]
    assert job["attempt_files_after_sha256"]==registration["attempt_files"]
    assert job["native_files_after_sha256"]==registration["native_files"]
    job["all_source_files_after_exact"]=all(sha(ATTEMPT/"source"/name)==expected for name,expected in source["files"].items())
    save(ledger,job)
print(json.dumps({k:job.get(k) for k in ("status","wall_seconds","first_failure","all_source_files_after_exact")}),flush=True)
raise SystemExit(0 if job["status"]=="completed_incomplete_prefix_only" else 1)
