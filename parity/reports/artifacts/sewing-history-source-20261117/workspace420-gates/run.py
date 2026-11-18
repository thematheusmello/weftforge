import time
STARTED = time.monotonic()
import hashlib
import json
import os
from pathlib import Path
import re
import resource
import subprocess
import sys
import resources as r

ROOT = Path(__file__).resolve().parent
WT = ROOT.parent / 'positive-target-sewing-component-20261117'
DEADLINE = STARTED + 1200.
RSS_CAP = 6442450944
OUTPUT_CAP = 16777216
BASE_SOURCE_COMMIT = 'd982d3af570e862fc8a5d059a32256eb6a45d1ce'
BASE_SOURCE88_SHA256 = '900c36680a7f6b68d175014c62ab85e2d27ed9113ceb0043435a6c4cb96657c5'
ALLOWED_CHANGED_FILES = ['crates/weftforge-sim/src/constraints.rs', 'crates/weftforge-sim/src/tests.rs', 'docs/physics.md']


def read(path):
    return json.loads(Path(path).read_text())


def persist(name, value):
    path = ROOT / name
    r.write_new(path, r.encoded(value))
    with path.open('rb') as stream:
        os.fsync(stream.fileno())


def output_bytes():
    # Runtime logs/receipts are create/append-only; no model export directory is counted here.
    return sum(q.stat().st_size for q in r.files_below(ROOT/'runtime'))


def compiler_inventory():
    return sorted(str(q.relative_to(WT)) for name in ['apps','crates','vendor','xtask']
                  for q in (WT/name).rglob('*') if q.is_file() and (q.suffix == '.rs' or q.name == 'Cargo.toml'))


def verify(freeze, freeze_sha, go_sha):
    r.require(r.sha(ROOT/'FREEZE.json',DEADLINE) == freeze_sha and r.sha(ROOT/'ROOT-GO.json',DEADLINE) == go_sha, 'Root freeze or one-run GO changed')
    r.require(set(freeze['controller_inputs']) == set(['COMPOSITION.json', 'INDEPENDENT-SOURCE-REVIEW.json', 'RESOURCE-PREFLIGHT.json', 'ROOT-SOURCE-BINDING.json', 'TEST-COMMAND-SCOPE.json', 'resources.py', 'run.py']), 'Controller approval/input roster changed')
    binding=read(ROOT/'ROOT-SOURCE-BINDING.json')
    r.require(r.sha(ROOT/'ROOT-SOURCE-BINDING.json',DEADLINE) == freeze['source_review_sha256'], 'Genuine root source binding changed')
    r.require(binding['root_source_approved'] is True and binding['base_source_commit'] == BASE_SOURCE_COMMIT and binding['commit'] == freeze['commit'] and binding['source88'] == freeze['source88'] and binding['expected_summary_count'] == freeze['expected_summary_count'] and binding['expected_workspace_test_total'] == freeze['expected_workspace_test_total'], 'Final source/test count lacks genuine root approval')
    r.require(r.sha(ROOT/'INDEPENDENT-SOURCE-REVIEW.json',DEADLINE) == binding['independent_source_review_sha256'], 'Independent final source review changed')
    composition=read(ROOT/'COMPOSITION.json')
    r.require(r.sha(ROOT/'COMPOSITION.json',DEADLINE) == freeze['composition_sha256'], 'Composition binding changed')
    r.require(composition['root_source_approved'] is True and composition['base_source_commit'] == BASE_SOURCE_COMMIT and composition['candidate_commit'] == freeze['commit'] and composition['source88'] == freeze['source88'], 'Composition lacks final root approval')
    r.require(hashlib.sha256(r.encoded(composition['base_source88'])).hexdigest() == BASE_SOURCE88_SHA256 and composition['allowed_changed_files'] == ALLOWED_CHANGED_FILES, 'Exact d982 base/three-file scope changed')
    r.require(set(composition['base_source88']) == set(freeze['source88']) and sorted(name for name in freeze['source88'] if freeze['source88'][name] != composition['base_source88'][name]) == ALLOWED_CHANGED_FILES, 'Expected only three sewing/normalization changes; other85 including main72219 must remain exact')
    r.require(freeze['worktree'] == str(WT) and freeze['child_env']['CARGO_TARGET_DIR'] == str(WT/'target') and freeze['target_cache_policy'] == {'path':str(WT/'target'),'warm_own_target':True,'cold_own_target':False,'compiled_cache_copied':False}, 'Require warm own-worktree target; no cross-copy')
    r.no_links(WT/'target',WT)
    r.require((WT/'target').is_dir(), 'Expected existing own-worktree target')
    r.require(subprocess.check_output(['/usr/bin/git','rev-parse','HEAD'],cwd=WT,text=True).strip() == freeze['commit'], 'Candidate HEAD changed')
    r.require(subprocess.check_output(['/usr/bin/git','diff','--name-only',BASE_SOURCE_COMMIT,freeze['commit'],'--'],cwd=WT,text=True).splitlines() == ALLOWED_CHANGED_FILES, 'Full tracked-tree diff exceeds exact three-file scope')
    r.require(subprocess.check_output(['/usr/bin/git','status','--porcelain=v1','--untracked-files=no'],cwd=WT,text=True) == '', 'Candidate tracked source no longer clean')
    for group, count in [('source88',88),('metadata_frozen',59),('original8',8)]:
        r.require(len(freeze[group]) == count, 'Input roster count changed')
        for name, digest in freeze[group].items():
            r.require(r.sha(WT/name,DEADLINE) == digest, group+' changed: '+name)
    for name,digest in freeze['controller_inputs'].items():
        r.require(r.sha(ROOT/name,DEADLINE) == digest, 'Controller input changed: '+name)
    for name,mode in freeze['input_modes'].items():
        r.require((WT/name).stat().st_mode & 0o777 == mode, 'Input mode changed: '+name)
    for tool in freeze['tools'].values():
        r.require(r.sha(tool['path'],DEADLINE) == tool['sha256'], 'Actual tool changed')
    for name,digest in freeze['cargo_config_inputs'].items():
        path=Path(name)
        r.require((r.sha(path,DEADLINE) if path.is_file() else None) == digest, 'Cargo config changed: '+name)
    r.require(compiler_inventory() == freeze['compiler_inventory'] and not (WT/'build.rs').exists(), 'Compiler source inventory changed')
    r.deadline_check(DEADLINE)
    return {'all_source88_metadata59_original8_exact':True,'clean_exact_commit':True,'tools_configs_compiler_inventory_exact':True}


def main():
    r.require(sys.flags.optimize == 0 and sys.dont_write_bytecode and os.environ.get('PYTHONOPTIMIZE') in (None,'0'), 'Use Python optimization0 and -B')
    freeze=read(ROOT/'FREEZE.json')
    r.require(freeze['draft_only'] is False and freeze['root_source_approved'] is True and isinstance(freeze['commit'],str) and re.fullmatch(r'[0-9a-f]{40}',freeze['commit']) is not None, 'Final clean source must be genuinely root-approved')
    r.require(type(freeze['expected_workspace_test_total']) is int and freeze['expected_workspace_test_total'] > 0 and type(freeze['expected_summary_count']) is int and freeze['expected_summary_count'] > 0, 'Final expected test count must be derived/reviewed, not a pending hint')
    freeze_sha=r.sha(ROOT/'FREEZE.json',DEADLINE)
    go_sha=r.sha(ROOT/'ROOT-GO.json',DEADLINE)
    r.require(read(ROOT/'ROOT-GO.json') == {'status':'authorized_one_workspace_gate_run','explicit_root_review_and_go':True,'commit':freeze['commit'],'source_freeze_sha256':freeze_sha,'source_review_sha256':freeze['source_review_sha256']}, 'Distinct exact one-run root GO required')
    r.require(not (ROOT/'runtime').exists(), 'One source-gate command only; no retry')
    (ROOT/'runtime').mkdir()
    persist('runtime/STARTED.json',{'pid':os.getpid(),'argv':sys.argv,'cwd':str(WT),'started_monotonic':STARTED,'deadline':DEADLINE,'source_freeze_sha256':r.sha(ROOT/'FREEZE.json'),'caps':freeze['caps'],'root_go_sha256':go_sha,'source_review_sha256':freeze['source_review_sha256'],'target_cache_policy':freeze['target_cache_policy']})
    stages=[];errors=[];summary=None;binaries={};final_guard=None
    try:
        verify(freeze,freeze_sha,go_sha)
        status=os.statvfs(WT)
        r.require(int(status.f_bavail)*int(status.f_frsize)>=8000000000,'Disk below8GB before gates')
        for key in freeze['rejected_ambient_keys']:
            r.require(os.environ.get(key) in (None,''), 'Unexpected ambient compiler wrapper/flags: '+key)
        env=dict(os.environ)
        for key in ['CARGO_BUILD_RUSTC_WRAPPER','CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER']:
            env.pop(key,None)
        env.update(freeze['child_env'])
        for entry in freeze['commands']:
            name,argv=entry['name'],entry['argv']
            pre=verify(freeze,freeze_sha,go_sha)
            phase_started=time.monotonic()
            def on_start(value,n=name,started=phase_started):
                value['stage_start_monotonic_seconds']=started
                value['at_utc']=__import__('datetime').datetime.now(__import__('datetime').timezone.utc).isoformat()
                persist('runtime/'+n+'-STARTED.json',value)
                print(json.dumps({'stage_started':n,'pid':value['pid'],'process_group':value['process_group']}),flush=True)
            stage=r.monitor(argv,WT,ROOT/('runtime/'+name+'.stdout'),ROOT/('runtime/'+name+'.stderr'),
                DEADLINE,output_bytes,rss_cap=RSS_CAP,output_cap=OUTPUT_CAP,reserve=262144,child_env=env,on_start=on_start,disk_floor=8000000000)
            stage.update({'name':name,'pre_guard':pre,'stage_start_monotonic_seconds':phase_started,'stage_elapsed_monitor_seconds':stage['end_monotonic_seconds']-phase_started})
            stages.append(stage)
            try:
                stage['post_guard']=verify(freeze,freeze_sha,go_sha)
                stage['all_source88_metadata59_original8_exact']=True
                r.require(stage['exit_code']==0 and stage['stop_reason'] is None, name+' failed')
                if name=='workspace-tests':
                    text=(ROOT/('runtime/'+name+'.stdout')).read_text()+'\n'+(ROOT/('runtime/'+name+'.stderr')).read_text()
                    matches=re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;',text)
                    summary={'summary_count':len(matches),'passed':sum(int(x[1]) for x in matches),'failed':sum(int(x[2]) for x in matches),'ignored':sum(int(x[3]) for x in matches),'all_status_ok':all(x[0]=='ok' for x in matches),'raw_summaries':[{'status':x[0],'passed':int(x[1]),'failed':int(x[2]),'ignored':int(x[3])} for x in matches]}
                    stage['workspace_test_summary']=summary
                    r.require(summary['summary_count']==freeze['expected_summary_count'] and summary['passed']==freeze['expected_workspace_test_total'] and summary['failed']==0 and summary['ignored']==0 and summary['all_status_ok'], 'Unexpected workspace test summary')
            finally:
                persist('runtime/'+name+'-RESULT.json',stage)
            print(json.dumps({'stage_completed':name,'exit_code':stage['exit_code'],'source88_metadata59_original8_exact':True}),flush=True)
        for name in ['weftforge','weftforge-cli']:
            path=WT/'target/debug'/name
            r.require(path.is_file(), 'Missing built executable: '+name)
            binaries[name]={'path':str(path),'sha256':r.sha(path,DEADLINE),'bytes':path.stat().st_size,'mode':path.stat().st_mode & 0o777,'built_not_launched_by_this_controller':True}
        final_guard=verify(freeze,freeze_sha,go_sha)
        status=os.statvfs(WT)
        r.require(int(status.f_bavail)*int(status.f_frsize)>=8000000000,'Disk below8GB at final guard')
    except BaseException as error:
        errors.append(type(error).__name__+': '+str(error))
    self_rss=int(resource.getrusage(resource.RUSAGE_SELF).ru_maxrss)
    if self_rss>RSS_CAP:errors.append('Supervisor exact RSS cap')
    elapsed=time.monotonic()-STARTED
    if elapsed>=1200.:errors.append('Whole source-gate clock cap')
    if output_bytes()+262144>=OUTPUT_CAP:errors.append('Whole retained output cap')
    result={'commit':freeze['commit'],'status':'GREEN_FOUR_WORKSPACE_SOURCE_GATES' if not errors else 'FAILED_WORKSPACE_SOURCE_GATES','all4_stages_passed':not errors and len(stages)==4,'errors':errors,'stages':stages,'workspace_test_summary':summary,'built_binaries':binaries,'final_guard':final_guard,'source88':freeze['source88'],'metadata_frozen':freeze['metadata_frozen'],'original8':freeze['original8'],'source_freeze_sha256':freeze_sha,'root_go_sha256':go_sha,'source_review_sha256':freeze['source_review_sha256'],'composition_sha256':freeze['composition_sha256'],'elapsed_before_final_persistence_seconds':elapsed,'supervisor_exact_peak_RSS_bytes':self_rss,'retained_output_bytes_before_result':output_bytes(),'no_xtask_registry_refresh_native_launch_fullgarment_or_speed_acceptance':True,'observation_qualification':freeze['observation_qualification']}
    persist('runtime/RESULT.json',result)
    persist('runtime/TERMINAL.json',{'result_sha256':r.sha(ROOT/'runtime/RESULT.json'),'all_direct_children_reaped':True,'stage_PIDs':[stage['pid'] for stage in stages],'elapsed_after_result_persistence_seconds':time.monotonic()-STARTED,'source_freeze_sha256':r.sha(ROOT/'FREEZE.json')})
    terminal_sha=r.sha(ROOT/'runtime/TERMINAL.json',DEADLINE)
    volume=output_bytes()
    elapsed=time.monotonic()-STARTED
    accepted=not errors and elapsed<1200. and volume+65536<OUTPUT_CAP and self_rss<=RSS_CAP
    print(json.dumps({'status':result['status'],'controller_exit_code':0 if accepted else 1,'command_complete_under_caps':accepted,'elapsed_after_all_persistence_seconds':elapsed,'retained_output_bytes':volume,'terminal_sha256':terminal_sha,'no_garment_or_speed_acceptance':True}),flush=True)
    return 0 if accepted else 1


if __name__=='__main__':
    raise SystemExit(main())
