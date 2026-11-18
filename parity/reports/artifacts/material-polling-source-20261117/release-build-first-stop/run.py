import time
STARTED = time.monotonic()
import json
import os
from pathlib import Path
import resource
import subprocess
import sys
import resources as r

ROOT=Path(__file__).resolve().parent
WT=ROOT.parent/'material-polling-release-20261117'
DEADLINE=STARTED+1200.
RSS_CAP=6442450944
OUTPUT_CAP=16777216


def read(path):
    return json.loads(Path(path).read_text())


def persist(name,value):
    path=ROOT/name
    r.write_new(path,r.encoded(value))
    with path.open('rb') as stream:
        os.fsync(stream.fileno())


def output_bytes():
    return sum(q.stat().st_size for q in r.files_below(ROOT/'runtime'))


def compiler_inventory():
    return sorted(str(q.relative_to(WT)) for name in ['apps','crates','vendor','xtask']
                  for q in (WT/name).rglob('*') if q.is_file() and (q.suffix=='.rs' or q.name=='Cargo.toml'))


def verify(freeze):
    r.require(subprocess.check_output(['/usr/bin/git','rev-parse','HEAD'],cwd=WT,text=True).strip()==freeze['commit'],'Release source HEAD changed')
    r.require(subprocess.check_output(['/usr/bin/git','status','--porcelain=v1','--untracked-files=no'],cwd=WT,text=True)=='','Release tracked source dirty')
    for group,count in [('source88',88),('metadata_frozen',59),('original8',8)]:
        r.require(len(freeze[group])==count,'Roster count changed')
        for name,digest in freeze[group].items():
            r.require(r.sha(WT/name,DEADLINE)==digest,group+' changed: '+name)
    for name,mode in freeze['input_modes'].items():
        r.require((WT/name).stat().st_mode & 0o777==mode,'Input mode changed: '+name)
    for name,digest in freeze['controller_inputs'].items():
        r.require(r.sha(ROOT/name,DEADLINE)==digest,'Build controller input changed')
    for tool in freeze['tools'].values():
        r.require(r.sha(tool['path'],DEADLINE)==tool['sha256'],'Actual build tool changed')
    for name,digest in freeze['cargo_config_inputs'].items():
        q=Path(name)
        r.require((r.sha(q,DEADLINE) if q.is_file() else None)==digest,'Cargo config changed')
    for name,digest in freeze['preserved_seals'].items():
        r.require(r.sha(name,DEADLINE)==digest,'Old source/gate/comparison seal changed')
    r.require(compiler_inventory()==freeze['compiler_inventory'] and not (WT/'build.rs').exists(),'Compiler source inventory changed')
    r.deadline_check(DEADLINE)
    return {'all_source88_metadata59_original8_exact':True,'clean_exact_commit':True,'compiler_inventory_tools_configs_old_seals_exact':True}


def main():
    r.require(sys.flags.optimize==0 and sys.dont_write_bytecode and os.environ.get('PYTHONOPTIMIZE') in (None,'0'),'Use Python optimization0 and -B')
    f=read(ROOT/'FREEZE.json')
    r.require(not (ROOT/'runtime').exists(),'One build-only command, no retry')
    r.require(not (WT/'target').exists(),'New worktree target must be cold')
    r.require(f['host_memory_before_launch']['budget_feasible'],'Host memory preflight does not support6GiB cap')
    (ROOT/'runtime').mkdir()
    persist('runtime/STARTED.json',{'pid':os.getpid(),'argv':sys.argv,'cwd':str(WT),'started_monotonic':STARTED,'deadline':DEADLINE,'source_freeze_sha256':r.sha(ROOT/'FREEZE.json'),'caps':f['caps'],'cold_own_target_before_build':True})
    stage=None;errors=[];binaries={};final_guard=None
    try:
        pre=verify(f)
        for key in f['rejected_ambient_keys']:
            r.require(os.environ.get(key) in (None,''),'Unexpected ambient compiler flag/wrapper: '+key)
        env=dict(os.environ)
        for key in ['CARGO_BUILD_RUSTC_WRAPPER','CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER']:
            env.pop(key,None)
        env.update(f['child_env'])
        def on_start(value):
            persist('runtime/release-build-STARTED.json',value)
            print(json.dumps({'build_started':True,'pid':value['pid'],'process_group':value['process_group'],'cold_own_target':True}),flush=True)
        stage=r.monitor(f['build_argv'],WT,ROOT/'runtime/release-build.stdout',ROOT/'runtime/release-build.stderr',DEADLINE,output_bytes,rss_cap=RSS_CAP,output_cap=OUTPUT_CAP,reserve=262144,child_env=env,on_start=on_start)
        stage['pre_guard']=pre
        try:
            stage['post_guard']=verify(f)
            stage['all_source88_metadata59_original8_exact']=True
            r.require(stage['exit_code']==0 and stage['stop_reason'] is None,'Release build failed')
        finally:
            persist('runtime/release-build-RESULT.json',stage)
        for name in ['weftforge','weftforge-cli']:
            q=WT/'target/release'/name
            r.require(q.is_file(),'Missing built release binary: '+name)
            binaries[name]={'path':str(q),'sha256':r.sha(q,DEADLINE),'bytes':q.stat().st_size,'mode':q.stat().st_mode & 0o777,'built_not_launched':True}
        final_guard=verify(f)
    except BaseException as error:
        errors.append(type(error).__name__+': '+str(error))
    self_rss=int(resource.getrusage(resource.RUSAGE_SELF).ru_maxrss)
    if self_rss>RSS_CAP:errors.append('Supervisor exact RSS cap')
    if time.monotonic()-STARTED>=1200.:errors.append('Whole release-build clock cap')
    if output_bytes()+262144>=OUTPUT_CAP:errors.append('Whole retained output cap')
    result={'commit':f['commit'],'status':'GREEN_NATIVE_CLI_RELEASE_BUILD' if not errors else 'FAILED_NATIVE_CLI_RELEASE_BUILD','release_build_passed':not errors,'errors':errors,'stage':stage,'release_binaries':binaries,'final_guard':final_guard,'source88':f['source88'],'metadata_frozen':f['metadata_frozen'],'original8':f['original8'],'source_freeze_sha256':r.sha(ROOT/'FREEZE.json'),'elapsed_before_final_persistence_seconds':time.monotonic()-STARTED,'supervisor_exact_peak_RSS_bytes':self_rss,'retained_output_bytes_before_result':output_bytes(),'profile':'Unmodified workspace profile.release thinLTO','build_launches':1 if stage else 0,'tests_models_benchmarks_native_GUI_or_bundle_creation':0,'observation_qualification':f['observation_qualification']}
    persist('runtime/RESULT.json',result)
    persist('runtime/TERMINAL.json',{'result_sha256':r.sha(ROOT/'runtime/RESULT.json'),'all_direct_children_reaped':True,'build_PID':stage['pid'] if stage else None,'elapsed_after_result_persistence_seconds':time.monotonic()-STARTED,'source_freeze_sha256':r.sha(ROOT/'FREEZE.json')})
    elapsed=time.monotonic()-STARTED;volume=output_bytes();accepted=not errors and elapsed<1200. and volume+65536<OUTPUT_CAP and self_rss<=RSS_CAP
    print(json.dumps({'status':result['status'],'controller_exit_code':0 if accepted else 1,'command_complete_under_caps':accepted,'elapsed_after_all_persistence_seconds':elapsed,'retained_output_bytes':volume,'terminal_sha256':r.sha(ROOT/'runtime/TERMINAL.json'),'build_only_no_execution_acceptance':True}),flush=True)
    return 0 if accepted else 1


if __name__=='__main__':
    raise SystemExit(main())
