"""Persistent launcher for one clean metadata cargo xtask verify; no retries."""
import argparse,hashlib,json,os,signal,subprocess,time
from pathlib import Path
B=Path(__file__).resolve().parent
p=argparse.ArgumentParser();p.add_argument('--go-sha256',required=True);a=p.parse_args()
raw=(B/'ROOT-GO.json').read_bytes()
if hashlib.sha256(raw).hexdigest()!=a.go_sha256:raise RuntimeError('ROOT-GO differs before launch')
g=json.loads(raw)
if hashlib.sha256((B/g['plan']).read_bytes()).hexdigest()!=g['plan_sha256']:raise RuntimeError('Authorized plan differs before launch')
started=time.monotonic();deadline=started+360

def write(name,value):
    data=(json.dumps(value,indent=2,sort_keys=True)+'\n').encode()
    fd=os.open(B/name,os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600)
    with os.fdopen(fd,'wb') as stream:stream.write(data);stream.flush();os.fsync(stream.fileno())

env=dict(os.environ);env.update(g['pinned_env'])
with (B/'controller.stdout').open('xb') as out,(B/'controller.stderr').open('xb') as err:
    child=subprocess.Popen(['<PYENV_HOME>/versions/3.13.0/bin/python3.13','-B',str(B/'runfull.py'),'verify-001'],cwd=B,env=env,stdout=out,stderr=err,start_new_session=True)
    launch={'launcher_PID':os.getpid(),'controller_PID':child.pid,'controller_PG':child.pid,'metadata_commit':g['metadata_commit'],'started_monotonic':started,'deadline':deadline,'root_go_sha256':a.go_sha256,'counts':{'workspace_source_inputs':93,'locked_registry_archives':410,'installed_metadata_files':231},'stdio_in_owned_files':True}
    write('LAUNCH.json',launch);print(json.dumps({'launched':launch}),flush=True)
    timed_out=False
    try:code=child.wait(timeout=max(.001,deadline-time.monotonic()))
    except subprocess.TimeoutExpired:
        timed_out=True
        for path in sorted((B/'verify-001').glob('*-STARTED.json')):
            try:os.killpg(json.loads(path.read_text())['pid'],signal.SIGKILL)
            except ProcessLookupError:pass
        try:os.killpg(child.pid,signal.SIGKILL)
        except ProcessLookupError:pass
        code=child.wait()
    out.flush();err.flush();os.fsync(out.fileno());os.fsync(err.fileno())
lines=(B/'controller.stdout').read_text().splitlines();terminal=json.loads(lines[-1]) if lines else {}
write('EXTERNAL-TERMINAL.json',{'controller_exit_code':code,'launcher_timeout':timed_out,'controller_terminal':terminal,'controller_stdout_sha256':hashlib.sha256((B/'controller.stdout').read_bytes()).hexdigest(),'controller_stderr_sha256':hashlib.sha256((B/'controller.stderr').read_bytes()).hexdigest(),'metadata_commit':g['metadata_commit'],'started_monotonic':started,'deadline':deadline})
# The final clock is sampled only after terminal/stdout/stderr fsync; its witness is post-case preservation.
closed=time.monotonic();volume=sum(x.stat().st_size for x in B.rglob('*') if x.is_file());disk=os.statvfs(B);available=disk.f_bavail*disk.f_frsize
accepted=code==0 and terminal.get('protocol_complete_under_caps') is True and not timed_out and closed<deadline and volume<=8388608 and available>=8000000000
witness={'accepted':accepted,'controller_exit_code':code,'launcher_timeout':timed_out,'actual_seconds_through_persisted_external_terminal':closed-started,'case_cap_seconds':360,'retained_bytes_at_terminal':volume,'retained_cap_bytes':8388608,'available_bytes':available,'external_terminal_sha256':hashlib.sha256((B/'EXTERNAL-TERMINAL.json').read_bytes()).hexdigest(),'qualification':'Clock sampled after external terminal and controller stdio fsync. This witness preserves that completed-case observation and is not a new command or timing regrade.'}
write('POST-TERMINAL-CLOCK.json',witness);print(json.dumps(witness),flush=True)
raise SystemExit(0 if accepted else 1)
