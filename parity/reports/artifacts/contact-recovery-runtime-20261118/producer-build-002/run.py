import time
START=time.monotonic()
import hashlib,json,os,resource,signal,subprocess,sys
from pathlib import Path
B=Path(__file__).resolve().parent
P=json.loads((B/'PLAN.json').read_text())
DEADLINE=START+120.
assert sys.flags.optimize==0 and sys.dont_write_bytecode
assert hashlib.sha256(Path(__file__).read_bytes()).hexdigest()==P['driver_sha256']
W=Path(P['worktree']);PACKET=Path(P['packet'])
assert PACKET==W/'.local/harbor-contact-recovery-preparation-20261118'
assert subprocess.check_output(['/usr/bin/git','rev-parse','HEAD'],cwd=W,text=True).strip()==P['source_commit']
assert subprocess.check_output(['/usr/bin/git','status','--porcelain=v1'],cwd=W,text=True)==''
for fd,name in [(1,'owner.stdout'),(2,'owner.stderr')]:
 f=os.open(B/name,os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600);os.dup2(f,fd);os.close(f)
def put(name,value):
 with (B/name).open('x') as f:json.dump(value,f,indent=2,sort_keys=True);f.write('\n');f.flush();os.fsync(f.fileno())
disk_samples=[]
def timeout(_signal,_frame):
 now=time.monotonic();disk=os.statvfs(W).f_bavail*os.statvfs(W).f_frsize;disk_samples.append([now,disk])
 if now>=DEADLINE:raise TimeoutError('Whole producer-build120s deadline')
 if disk<8000000000:raise RuntimeError('Available disk below8GB floor')
put('STARTED.json',{'pid':os.getpid(),'pgid':os.getpgrp(),'started_monotonic':START,'deadline':DEADLINE,'plan_sha256':hashlib.sha256((B/'PLAN.json').read_bytes()).hexdigest()})
old=signal.signal(signal.SIGALRM,timeout)
timeout(None,None)
signal.setitimer(signal.ITIMER_REAL,min(.1,max(.000001,DEADLINE-time.monotonic())),.1)
error=None;receipt=None
try:
 assert os.environ['PATH']==P['approved_parent_path']
 sys.path.insert(0,str(PACKET))
 import common as b
 import build_release
 assert b.SOURCE==P['source_commit']
 assert b.r.sha(PACKET/P['go_file'])==P['go_sha256']
 sys.argv=[str(PACKET/'build_release.py'),'--go',P['go_file'],'--go-sha256',P['go_sha256']]
 assert build_release.main()==0
 receipt=b.read(PACKET/'checks/BUILD-RECEIPT.json')
 assert receipt['stage']['exit_code']==0 and receipt['stage']['stop_reason'] is None
 assert b.r.sha(W/receipt['binary']['path'])==receipt['binary']['sha256']
 assert receipt['Cloth_initializations']==receipt['Cloth_steps']==0
except BaseException as e:error=type(e).__name__+': '+str(e)
finally:
 signal.setitimer(signal.ITIMER_REAL,0);signal.signal(signal.SIGALRM,old)
peak=resource.getrusage(resource.RUSAGE_SELF).ru_maxrss
volume=sum(p.stat().st_size for directory in [B,PACKET/'checks'] if directory.exists() for p in directory.rglob('*') if p.is_file())
disk=os.statvfs(W).f_bavail*os.statvfs(W).f_frsize
result={'source_commit':P['source_commit'],'error':error,'helper_returned_success':error is None,'whole_root_started_before_helper_preflight':True,'root_disk_samples':disk_samples,'root_exact_peak_rss_bytes':peak,'inner_aggregate_sampled_peak_rss_bytes':None if receipt is None else receipt['stage']['sampled_peak_rss_bytes'],'retained_build_text_bytes':volume,'available_disk_bytes':disk,'original_build_receipt_sha256':None if receipt is None else hashlib.sha256((PACKET/'checks/BUILD-RECEIPT.json').read_bytes()).hexdigest(),'binary':None if receipt is None else receipt['binary'],'Cloth_initializations':0,'Cloth_steps':0,'qualification':'Original frozen build_release.main executes in this same process; standard periodic SIGALRM samples disk and bounds whole120s including its preflight. Its proven monitor accounts Cargo group plus this supervisor RSS. No source/target/executable copied.'}
put('RESULT.json',result)
put('TERMINAL.json',{'result_sha256':hashlib.sha256((B/'RESULT.json').read_bytes()).hexdigest(),'error':error,'helper_and_Cargo_terminal':receipt is not None})
for n in ['owner.stdout','owner.stderr']:
 with (B/n).open('rb') as f:os.fsync(f.fileno())
s=os.statvfs(W);final_volume=sum(p.stat().st_size for directory in [B,PACKET/'checks'] if directory.exists() for p in directory.rglob('*') if p.is_file())
terminal_hash=hashlib.sha256((B/'TERMINAL.json').read_bytes()).hexdigest()
end=time.monotonic()
accepted=error is None and end<DEADLINE and peak<=1073741824 and final_volume<=8388608 and s.f_bavail*s.f_frsize>=8000000000
print(json.dumps({'accepted_through_prior_persistence':accepted,'whole_seconds':end-START,'cap_seconds':120,'terminal_sha256':terminal_hash,'retained_bytes':final_volume,'error':error}),flush=True);os.fsync(1)
raise SystemExit(0 if accepted else 1)
