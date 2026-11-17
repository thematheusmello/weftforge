from pathlib import Path
import datetime,hashlib,json,os,stat,subprocess
p=Path(__file__).resolve().parent;wt=p.parent.parent;repo=wt.parent.parent
sha=lambda q:hashlib.sha256(Path(q).read_bytes()).hexdigest()
git=lambda *a:subprocess.check_output(['/usr/bin/git',*a],cwd=wt,text=True)
def require(ok,msg):
    if not ok:raise RuntimeError(msg)
def save(name,d): (p/name).write_text(json.dumps(d,indent=2,sort_keys=True)+'\n')
base='52099b6f6b7aff43b3e9ada8bad4f830cb987134';require(git('rev-parse','HEAD').strip()==base,'Base mismatch')
original=git('show',base+':crates/weftforge-sim/src/constraints.rs');current=(wt/'crates/weftforge-sim/src/constraints.rs').read_text()
helper=current[current.index('fn material_cancellation('):current.index('pub(crate) fn solve_material(')]
unit=current[current.index('#[cfg(test)]\nmod material_cancellation_tests'):current.index('fn material_dot(')]
entry='    check_cancel(cancelled)?;\n    let original_cancelled = cancelled;\n    let material_cancelled = material_cancellation(original_cancelled);\n    let cancelled = &material_cancelled;\n'
blocks=[('use std::cell::Cell;\n',1),(helper,1),(entry,1),('        check_cancel(original_cancelled)?;\n',2),('            check_cancel(original_cancelled)?;\n',1),('    check_cancel(original_cancelled)?;\n',1),(unit,1)]
stripped=current
for text,count in blocks:
    if text.count('\n')==1:
        lines=stripped.splitlines(keepends=True);require(lines.count(text)==count,'Registered line count');stripped=''.join(line for line in lines if line!=text)
    else:
        require(stripped.count(text)==count,'Registered block count');stripped=stripped.replace(text,'')
require(stripped==original,'Non-polling production source changed')
oldtests=git('show',base+':crates/weftforge-sim/src/tests.rs');tests=(wt/'crates/weftforge-sim/src/tests.rs').read_text();oracle='fn material_network_reaches_independent_incremental_energy_equilibrium()'
require(tests[tests.index(oracle):]==oldtests[oldtests.index(oracle):],'Eight-face oracle changed')
save('REGISTERED-INSERTIONS.json',{'base_commit':base,'constraints_original_sha256':hashlib.sha256(original.encode()).hexdigest(),'constraints_candidate_sha256':sha(wt/'crates/weftforge-sim/src/constraints.rs'),'insertions':[{'text':t,'occurrences':n} for t,n in blocks],'removal_restores_exact_original_bytes':True,'eight_face_oracle_unchanged':True,'eight_face_oracle_source_sha256':hashlib.sha256(tests[tests.index(oracle):].encode()).hexdigest(),'PCG_residual':1e-12,'PCG_cap':32,'Armijo_cap':16,'successful_exits':'All four directly check original callback; accepted branch checks before both x/p copies, then only Ok'})
perf=repo/'.local/global-material-perf-probe-20261117/.local'
(p/'resources.py').write_bytes((perf/'material-kernel-clock-predicate-20261117/resources.py').read_bytes())
(p/'LOCKED-DEPENDENCIES.json').write_bytes((perf/'material-kernel-clock-predicate-20261117/LOCKED-DEPENDENCIES.json').read_bytes())
configs={}
for parent in [wt,*wt.parents,Path('<HOME>')]:
    for name in ['config','config.toml']:
        q=parent/'.cargo'/name;configs[str(q)]=sha(q) if q.is_file() else None
save('CARGO-CONFIG-INPUTS.json',configs)
protection={}
for path,expected in [(perf/'material-kernel-baseline-20261117/FINAL-MANIFEST.json','f145a89dbe45eedcd20671a96bfc2acb7bf26eea96c87433ea0fde339db3e8c4'),(perf/'material-kernel-clock-predicate-20261117/FINAL-MANIFEST.json','ca10761fc9345b83798e169cf813b74ac7c911e555062ceb843947c38ae78844'),(repo/'.local/global-material-full-case-closure-20261117/FINAL-MANIFEST.json','cfa30db44a98308aeaafdb1ee54a203740103c70011993ed5caf9aff0a4e9c00')]:
    require(sha(path)==expected,'Old seal changed');protection[str(path)]={'sha256':expected,'mode':stat.S_IMODE(path.stat().st_mode)}
    if path.parent.parent==perf:
        for name,item in json.loads(path.read_text())['files'].items(): require(sha(path.parent/name)==item['sha256'],'Old component artifact changed')
closed=repo/'.local/global-material-repair-20261117'
for name in ['crates/weftforge-sim/src/constraints.rs','crates/weftforge-sim/src/lib.rs','crates/weftforge-sim/src/tests.rs','docs/physics.md']:
    require((closed/name).read_bytes()==subprocess.check_output(['/usr/bin/git','show',base+':'+name],cwd=wt),'Closed source changed');protection[str(closed/name)]={'sha256':sha(closed/name),'mode':stat.S_IMODE((closed/name).stat().st_mode)}
save('PRESERVATION.json',protection)
(p/'DIFF.patch').write_text(git('diff','--','crates/weftforge-sim/src/constraints.rs','crates/weftforge-sim/src/tests.rs','docs/physics.md'))
save('PREPARATION-ERROR-001.json',{'scope':'Before any Cargo/check command','errors':['Initial evidence mkdir lacked parents=True; FileNotFoundError','Oracle locator incorrectly assumed test attribute immediately preceded function; ValueError before manifest write'],'corrections':['Created owned private parent','Locate exact unchanged function body; keep byte-equality guard'],'source_or_assertion_changes_from_errors':False,'Cargo_or_test_launches':0})
print(json.dumps({'strip_exact':True,'eight_face_oracle_unchanged':True,'own_target_absent':not (wt/'target').exists(),'diff_sha256':sha(p/'DIFF.patch')}))
