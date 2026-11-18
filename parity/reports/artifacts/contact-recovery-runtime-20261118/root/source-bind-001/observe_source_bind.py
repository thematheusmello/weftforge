"""One source-only helper invocation; no compiler, constructor, model or GUI."""
import time
START=time.monotonic()
import hashlib,json,os,resource,sys
from pathlib import Path
OUT=Path(__file__).resolve().parent
plan=json.loads((OUT/'INVOCATION-PLAN.json').read_text())
WT=Path(plan['cwd']);PACKET=Path(plan['argv'][2]).parent
sys.path.insert(0,str(PACKET))
import resources as r
DEADLINE=START+plan['wall_seconds']
assert plan['argv'][1]=='-B' and Path(plan['argv'][2]).name=='bind_source.py'
assert not (OUT/'STARTED.json').exists() and not (PACKET/'SOURCE-PREFLIGHT.json').exists()

def count():
    disk=os.statvfs(WT)
    r.require(disk.f_bavail*disk.f_frsize>=plan['disk_floor_bytes'],'Disk below original8GB floor')
    return sum(q.stat().st_size for q in OUT.rglob('*') if q.is_file())

def save(name,value):
    r.write_new(OUT/name,r.encoded(value))
    with (OUT/name).open('rb') as stream:os.fsync(stream.fileno())

env=dict(os.environ);env.update(plan['pinned_env'])
result=r.monitor(plan['argv'],WT,OUT/'stdout',OUT/'stderr',DEADLINE,count,
                 rss_cap=plan['RSS_cap_bytes'],output_cap=plan['output_cap_bytes'],
                 reserve=65536,child_env=env,
                 on_start=lambda h:save('STARTED.json',{'observer_pid':os.getpid(),'started':START,'deadline':DEADLINE,'plan_sha256':r.sha(OUT/'INVOCATION-PLAN.json'),**h}))
parent_peak=resource.getrusage(resource.RUSAGE_SELF).ru_maxrss
ok=result['exit_code']==0 and result['stop_reason'] is None and parent_peak+result['peak_rss_bytes']<=plan['RSS_cap_bytes']
preflight=PACKET/'SOURCE-PREFLIGHT.json'
receipt={'source_bind_only':True,'stage':result,'supervisor_peak_rss_bytes':parent_peak,'conservative_parent_plus_child_peak_bytes':parent_peak+result['peak_rss_bytes'],'source_preflight_sha256':r.sha(preflight) if preflight.is_file() else None,'source_preflight_written':preflight.is_file(),'Cargo_Cloth_model_GUI_calls':0,'build_numerical_GO_absent':not (PACKET/'ROOT-GO.json').exists(),'pinned_env':plan['pinned_env'],'child_argv':plan['argv'],'source_bind_invocations':1,'stdout_sha256':r.sha(OUT/'stdout'),'stderr_sha256':r.sha(OUT/'stderr')}
save('RESULT.json',receipt)
digest=r.sha(OUT/'RESULT.json');volume=count();closed=time.monotonic()
accepted=ok and preflight.is_file() and closed<DEADLINE and volume<=plan['output_cap_bytes']
witness={'accepted':accepted,'elapsed_through_result_persistence_seconds':closed-START,'wall_cap_seconds':plan['wall_seconds'],'result_sha256':digest,'retained_bytes':volume,'conservative_peak_bytes':receipt['conservative_parent_plus_child_peak_bytes'],'source_preflight_sha256':receipt['source_preflight_sha256'],'qualification':'Source-only observation. Actual helper never invokes Cargo or Cloth. No build or numerical GO.'}
save('EXTERNAL-WITNESS.json',witness);print(json.dumps(witness),flush=True)
raise SystemExit(0 if accepted else 1)
