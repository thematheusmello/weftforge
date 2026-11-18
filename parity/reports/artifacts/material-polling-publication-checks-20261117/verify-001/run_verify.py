"""One bounded clean xtask verify; no source412/416 or actual garment/native pipeline."""
import hashlib,json,os,subprocess,sys,time
from pathlib import Path
import resources as r
ROOT=Path(__file__).resolve().parent

def persist(name,value,deadline):
    r.deadline_check(deadline)
    p=ROOT/name;p.parent.mkdir(parents=True,exist_ok=True)
    r.write_new(p,r.encoded(value))
    with p.open('rb') as stream:os.fsync(stream.fileno())
    return r.sha(p,deadline)

def total_bytes():return sum(p.stat().st_size for p in r.files_below(ROOT))
def available(path):s=os.statvfs(path);return int(s.f_bavail)*int(s.f_frsize)
def cap_ok(clock,deadline,volume,volume_cap,rss,rss_cap,disk,disk_floor):
    return clock<deadline and volume<=volume_cap and rss<=rss_cap and disk>=disk_floor

def main():
    import argparse
    parser=argparse.ArgumentParser();parser.add_argument('--go',required=True);parser.add_argument('--go-sha256',required=True);parser.add_argument('--started',required=True,type=float);args=parser.parse_args()
    started=args.started;deadline=started+300.;r.deadline_check(deadline)
    go_path=ROOT/args.go;r.require(r.sha(go_path,deadline)==args.go_sha256,'ROOT-GO hash differs');go=json.loads(go_path.read_text());frozen_path=ROOT/go['freeze_path'];r.require(r.sha(frozen_path,deadline)==go['freeze_sha256'],'Input freeze differs');f=json.loads(frozen_path.read_text());wt=Path(f['worktree'])
    r.require(go['status']=='authorized_one_clean_xtask_verify' and go['one_cycle_only'] is True and go['metadata_commit']==f['metadata_commit'],'Wrong ROOT authorization')
    r.require(go['caps']=={'seconds':300,'RSS_bytes':6442450944,'retained_bytes':16777216,'disk_floor_bytes':8000000000},'Caps differ')
    for p,h in f['controller_sources'].items():r.require(r.sha(ROOT/p,deadline)==h,'Controller source changed')
    for p,h in f['project_tool_inputs'].items():r.require(r.sha(wt/p,deadline)==h,'Generator/schema tool input changed')
    for p,h in f['actual_tool_binaries'].items():r.require(r.sha(p,deadline)==h,'Actual pinned tool changed')
    def git(*a):
        r.deadline_check(deadline)
        return subprocess.check_output(['git',*a],cwd=wt,timeout=max(.001,deadline-time.monotonic()))
    def original_snapshot():
        result={}
        for s in f['selected_originals']:
            r.deadline_check(deadline)
            if s['original_kind']=='file':
                p=Path(s['original_path']);digest=r.sha(p,deadline);size=p.stat().st_size;mode=format(p.stat().st_mode&511,'04o')
            else:
                c,p=s['original_locator'].split(':',1);raw=git('show',c+':'+p);digest=hashlib.sha256(raw).hexdigest();size=len(raw);mode='0644'
            r.require(digest==s['original_sha256'] and size==s['original_bytes'] and mode==s['original_mode'],'Selected original changed')
            result[s['original_locator']]={'sha256':digest,'bytes':size,'mode':mode}
        return result
    def public_snapshot():
        result={}
        for p,v in f['public_artifacts'].items():
            full=wt/p;digest=r.sha(full,deadline);mode=format(full.stat().st_mode&511,'04o');size=full.stat().st_size
            r.require(digest==v['sha256'] and mode==v['mode'] and size==v['bytes'],'Public artifact changed')
            result[p]={'sha256':digest,'bytes':size,'mode':mode}
        return result
    def immutable_snapshot():
        actual={p:r.sha(wt/p,deadline) for p in f['immutable156']};r.require(actual==f['immutable156'],'One of156 immutable inputs changed');return actual
    def outputs_snapshot():return {p:{'sha256':r.sha(wt/p,deadline),'bytes':(wt/p).stat().st_size,'mode':format((wt/p).stat().st_mode&511,'04o')} for p in f['allowed_outputs']}
    r.require(git('rev-parse','HEAD').decode().strip()==f['metadata_commit'] and git('status','--porcelain')==b'','Need exact clean committed metadata')
    immutable_before=immutable_snapshot();originals_before=original_snapshot();public_before=public_snapshot();outputs_before=outputs_snapshot()
    persist('BEFORE.json',{'immutable156':immutable_before,'originals286':originals_before,'public287':public_before,'no_mutable_outputs':outputs_before,'HEAD':f['metadata_commit'],'index_clean':True},deadline)
    r.require(r.sha(ROOT/'REUSED-PURE-CAPS-SELFCHECKS.json',deadline)==go['reused_pure_caps_sha256'],'Existing exact-monitor/cap predicate proof changed')
    persist('RUN-ONCE.json',{'started':started,'deadline':deadline,'controller_PID':os.getpid(),'root_GO_sha256':args.go_sha256},deadline)
    runtime=ROOT/'runtime';runtime.mkdir(exist_ok=False)
    env=dict(os.environ);env.update(go['pinned_env'])
    def on_start(handle):
        persist('runtime/STARTED.json',handle|{'started':started,'deadline':deadline,'controller_PID':os.getpid(),'argv':go['argv'],'pinned_env':go['pinned_env']},deadline)
        print(json.dumps({'state':'xtask_running','Cargo_PID':handle['pid'],'controller_PID':os.getpid()}),flush=True)
    stage=r.monitor(go['argv'],wt,runtime/'stdout.log',runtime/'stderr.log',deadline,total_bytes,
        rss_cap=6442450944,output_cap=16777216,reserve=131072,on_start=on_start,child_env=env,disk_floor=8000000000)
    persist('STAGE.json',stage,deadline)
    # Postguards and their persistence consume the SAME original300s deadline.
    immutable_after=immutable_snapshot();originals_after=original_snapshot();public_after=public_snapshot();outputs_after=outputs_snapshot()
    dirty=git('diff','--name-only').decode().splitlines();head_fixed=git('rev-parse','HEAD').decode().strip()==f['metadata_commit'];index_fixed=git('diff','--cached','--name-only')==b''
    guards={'all156_immutable_exact':immutable_before==immutable_after,'all286_original_hashes_bytes_modes_exact':originals_before==originals_after,
      'all287_public_hashes_bytes_modes_exact':public_before==public_after,'HEAD_fixed':head_fixed,'index_empty_fixed':index_fixed,
      'no_working_paths_dirty':set(dirty)==set(f['allowed_outputs']),'checkout_clean_after':git('status','--porcelain')==b'','project_tool_inputs_fixed':all(r.sha(wt/p,deadline)==h for p,h in f['project_tool_inputs'].items()),
      'controller_sources_fixed':all(r.sha(ROOT/p,deadline)==h for p,h in f['controller_sources'].items())}
    persist('AFTER.json',{'immutable156':immutable_after,'originals286':originals_after,'public287':public_after,'no_mutable_outputs':outputs_after,'dirty':dirty,'guards':guards},deadline)
    candidate=stage['exit_code']==0 and stage['stop_reason'] is None and all(guards.values())
    root_result={'status':'pending_final_post_persistence_clock_volume_verdict','xtask_exit_code':stage['exit_code'],'stage':stage,'guards':guards,'candidate_success':candidate,
      'started':started,'deadline':deadline,'metadata_commit':f['metadata_commit'],'mutable_output_before':outputs_before,'mutable_output_after':outputs_after,
      'raw_stdout_sha256':r.sha(runtime/'stdout.log',deadline),'raw_stderr_sha256':r.sha(runtime/'stderr.log',deadline),'no_retry_or_source412_416_model_workflow_UI_network_execution':True}
    root_sha=persist('ROOT-RESULT.json',root_result,deadline)
    # b62b1 order: finish receipt fsync/hash, THEN volume/disk traversal, THEN final clock.
    volume=total_bytes();disk=available(wt);closed=time.monotonic()
    bounded=cap_ok(closed,deadline,volume,16777216,max(stage['peak_rss_bytes'] or 0,stage['sampled_peak_rss_bytes']),6442450944,disk,8000000000)
    accepted=candidate and bounded
    print(json.dumps({'state':'xtask_terminal','accepted':accepted,'xtask_exit_code':stage['exit_code'],'controller_exit_code':0 if accepted else 1,'ROOT_RESULT_sha256':root_sha,
      'started':started,'deadline':deadline,'final_elapsed_after_all_receipt_persistence_seconds':closed-started,'final_volume_before_clock_bytes':volume,'final_available_bytes':disk,
      'Cargo_PID':stage['pid'],'controller_PID':os.getpid(),'metadata_commit':f['metadata_commit'],'guards':guards}),flush=True)
    return 0 if accepted else 1
if __name__=='__main__':
    try:raise SystemExit(main())
    except SystemExit:raise
    except BaseException as error:
        path=ROOT/'FAILURE.json'
        if not path.exists():
            r.write_new(path,r.encoded({'classification':'Unexpected failure retained; no autonomous retry','error':type(error).__name__+': '+str(error),'at_monotonic':time.monotonic(),'controller_PID':os.getpid()}))
        raise
