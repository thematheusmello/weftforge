"""One execution plus automatic external persisted-clock witness; no numerical replay."""
import hashlib,json,os,signal,subprocess,sys,time
from pathlib import Path
OUT=Path(__file__).resolve().parent

def admitted(code,terminal,closed,deadline,volume):
    return code in (0,2) and terminal.get("root_operational_accepted") is True and type(terminal.get("strict_quality_passed")) is bool and (code==0)==terminal["strict_quality_passed"] and closed<deadline and volume<=268435456

assert admitted(2,{"root_operational_accepted":True,"strict_quality_passed":False},419,420,1)
assert not admitted(0,{"root_operational_accepted":True,"strict_quality_passed":True},420,420,1)
assert not admitted(0,{"root_operational_accepted":False,"strict_quality_passed":True},1,420,1)
assert not admitted(2,{"root_operational_accepted":True,"strict_quality_passed":True},1,420,1)
assert not admitted(0,{"root_operational_accepted":True,"strict_quality_passed":True},1,420,268435457)

def main():
    assert not (OUT/"EXTERNAL-START.json").exists(),"Once-only external launcher; no retry"
    admission=json.loads((OUT/"ROOT-ADMISSION.json").read_text())
    for name,key in [("supervise.py","outer_sha256"),("launch_and_record.py","external_launcher_sha256")]:
        assert hashlib.sha256((OUT/name).read_bytes()).hexdigest()==admission[key]
    packet=Path(admission["packet"])
    assert hashlib.sha256((packet/"ROOT-GO.json").read_bytes()).hexdigest()==admission["GO_sha256"]
    with (OUT/"outer.stdout").open("xb") as stdout,(OUT/"outer.stderr").open("xb") as stderr:
        child=subprocess.Popen([sys.executable,"-B",str(OUT/"supervise.py")],cwd=OUT,stdout=stdout,stderr=stderr,start_new_session=True)
        record={"external_PID":os.getpid(),"outer_PID":child.pid,"started":time.monotonic(),"admission_sha256":hashlib.sha256((OUT/"ROOT-ADMISSION.json").read_bytes()).hexdigest()}
        with (OUT/"EXTERNAL-START.json").open("x") as stream:json.dump(record,stream);stream.flush();os.fsync(stream.fileno())
        try:code=child.wait(timeout=450)
        except subprocess.TimeoutExpired:
            for path in [OUT/"RUNNING.json",*sorted((packet/"runtime").glob("*-started.json"))]:
                if path.exists():
                    handle=json.loads(path.read_text());pid=handle.get("pid",handle.get("process_group"))
                    if pid:
                        try:os.killpg(pid,signal.SIGKILL)
                        except ProcessLookupError:pass
            os.killpg(child.pid,signal.SIGKILL);child.wait();raise RuntimeError("Outer failed to terminate; no acceptance")
    terminal=json.loads((OUT/"outer.stdout").read_text())
    start=json.loads((OUT/"OUTER-START.json").read_text())
    record.update(outer_exit_code=code,outer_terminal=terminal,outer_stdout_sha256=hashlib.sha256((OUT/"outer.stdout").read_bytes()).hexdigest(),outer_stderr_sha256=hashlib.sha256((OUT/"outer.stderr").read_bytes()).hexdigest(),external_final_clock_verdict_pending=True)
    with (OUT/"EXTERNAL-TERMINAL.json").open("x") as stream:json.dump(record,stream,indent=2);stream.write("\n");stream.flush();os.fsync(stream.fileno())
    witness_sha=hashlib.sha256((OUT/"EXTERNAL-TERMINAL.json").read_bytes()).hexdigest()
    sys.path.insert(0,str(packet))
    import resources as r
    volume=r.directory_bytes((packet/name for name in r.OUTPUT_DIRECTORIES),shallow=(OUT,),deadline=start["deadline"])
    disk=os.statvfs(packet)
    assert disk.f_bavail*disk.f_frsize>=8000000000,"Available disk below original8GB floor after external persistence"
    closed=time.monotonic()
    result={"external_operational_accepted":admitted(code,terminal,closed,start["deadline"],volume),"strict_quality_passed":terminal["strict_quality_passed"],"outer_exit_code":code,"elapsed_after_external_witness_persistence_seconds":closed-start["started"],"total_counted_bytes":volume,"external_witness_sha256":witness_sha}
    print(json.dumps(result),flush=True)
    return code if result["external_operational_accepted"] else 1

if __name__=="__main__":raise SystemExit(main())
