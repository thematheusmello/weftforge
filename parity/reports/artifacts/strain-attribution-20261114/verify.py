"""Verify retained evidence only; default requires a clean Git-tracked publication checkout."""
import sys
sys.dont_write_bytecode = True
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess

BASE=Path(__file__).resolve().parent
SOURCE='6c45535cb312f7fed8ad872ee87b6ba5e5974e35'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
read=lambda p:json.loads(p.read_bytes())

def closed(directory,expected_manifest=None):
    path=directory/'MANIFEST.json'
    if expected_manifest:
        assert sha(path)==expected_manifest
    manifest=read(path)
    actual={p.relative_to(directory).as_posix() for p in directory.rglob('*') if p.is_file()}
    assert actual==set(manifest['files'])|{'MANIFEST.json'},str(directory.name)+' closure mismatch'
    for name,record in manifest['files'].items():
        file=directory/name
        assert not file.is_symlink() and not Path(name).is_absolute() and '..' not in Path(name).parts
        assert sha(file)==(record if isinstance(record,str) else record['sha256']),name
        if isinstance(record,dict):
            assert file.stat().st_size==record['bytes']
    return actual

def replay(script,*args):
    command=[sys.executable,'-B',str(script),*map(str,args)]
    result=subprocess.run(command,env={**os.environ,'PYTHONDONTWRITEBYTECODE':'1'},text=True,capture_output=True,check=True)
    assert not result.stderr,result.stderr
    return json.loads(result.stdout)

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--candidate-only',action='store_true',help='Preparation replay only: explicitly skip repository cleanliness/tracking checks; still verify all frozen artifacts and source snapshot hashes.')
    args=parser.parse_args()
    actual=closed(BASE)
    sensitive=[b'/'+p+b'/' for p in [b'Users',b'home',b'Volumes',b'private/var',b'var/folders']]
    for name in actual:
        path=BASE/name
        assert not path.is_symlink()
        assert not any(part in ['.git','target','__pycache__'] for part in Path(name).parts)
        data=path.read_bytes()
        assert not any(prefix in data for prefix in sensitive),name
        assert not re.search(rb'[A-Za-z]:\\Users\\',data),name
    provenance=read(BASE/'PROVENANCE.json')
    assert provenance['runtime_source_commit']==SOURCE
    frozen_manifests={'headless':'a5a81a5244f724fa4e7faba23abe03658e0d87fbe317486c5b6e8b199a302a8e','native':'115f4685dea9f915673cf016f6e748466cf621a8809ee953b0ed2aa5d3363734'}
    assert {s['bundle_subtree']:s['published_manifest_sha256'] for s in provenance['frozen_subtrees']}==frozen_manifests
    for subtree in provenance['frozen_subtrees']:
        assert subtree['all_subtree_bytes_identical'] and subtree['raw_manifest_sha256']==subtree['published_manifest_sha256']
        files=closed(BASE/subtree['bundle_subtree'],subtree['published_manifest_sha256'])
        assert len(files)==subtree['file_count_including_manifest']
    for entry in provenance['copied_origins']:
        assert sha(BASE/entry['bundle_path'])==entry['published_sha256']
        assert entry['byte_identical']==(entry['raw_sha256']==entry['published_sha256'])
        if not entry['byte_identical']:
            derivative=entry['derivative']
            assert derivative['kind']=='whole_workspace_pathname_prefix_only'
            assert derivative['replacement_count']>0 and derivative['roundtrip_byte_equality_checked']
            assert derivative['numeric_source_state_obj_png_bytes_changed'] is False
            assert entry['bundle_path'] in ['cli/runner-stdout','cli/jobs.json','cli/summary.json','gates/clippy.log','gates/tests.log','gates/package.log']
    assert sum(not e['byte_identical'] for e in provenance['copied_origins'])==6
    source=read(BASE/'headless/source-manifest.json')
    pretest=read(BASE/'binding/source-pretest.json')
    assert source['commit']==pretest['source_commit']==SOURCE and len(source['files'])==37
    assert source['files']==pretest['source_hashes']
    assert (BASE/'binding/source-pretest.json').read_bytes()==(BASE/'gates/source-pretest.json').read_bytes()==(BASE/'native/source-pretest.json').read_bytes()
    for name,expected in source['files'].items():
        assert sha(BASE/'headless/runtime-source'/name)==expected,name
    git_state='not checked: explicit candidate-only preparation replay'
    if not args.candidate_only:
        repository=Path(subprocess.check_output(['git','-C',str(BASE),'rev-parse','--show-toplevel'],text=True).strip())
        relative=BASE.relative_to(repository)
        assert not subprocess.check_output(['git','-C',str(repository),'status','--porcelain','--untracked-files=all'],text=True),'Publication checkout is not clean'
        tracked=set(subprocess.check_output(['git','-C',str(repository),'ls-files','-z','--',relative.as_posix()]).decode().split('\0'))-{''}
        assert tracked=={(relative/name).as_posix() for name in actual},'Evidence closure is not fully Git-tracked'
        scoped=set(subprocess.check_output(['git','-C',str(repository),'ls-files','-z','--',*pretest['source_scope']]).decode().split('\0'))-{''}
        assert scoped==set(source['files']),'Current source scope differs from frozen37 files'
        subprocess.run(['git','-C',str(repository),'merge-base','--is-ancestor',SOURCE,'HEAD'],check=True)
        for name,expected in source['files'].items():
            assert sha(repository/name)==expected,name
            committed=subprocess.check_output(['git','-C',str(repository),'show',SOURCE+':'+name])
            assert hashlib.sha256(committed).hexdigest()==expected,name
        assert sha(repository/'fixtures/grain-shirt.weftforge')=='82e7c6d398c7772d7e62c28196dcf11845020020d44fca9ac9113e615b9537e7'
        contract_file='spec/features/SIM-005-strain-attribution.md'
        frozen_contract=subprocess.check_output(['git','-C',str(repository),'show','209a8fda72aaca90a08c330427354dd457224a6c:'+contract_file])
        assert hashlib.sha256(frozen_contract).hexdigest()==sha(repository/contract_file)=='71bbea48b09a3f07cc516625e1506ae07a73a79f3db35fcf59564dc44cc23700'
        git_state='clean Git checkout; complete tracked evidence closure; current and committed37 source hashes equal6c455'
    contract='71bbea48b09a3f07cc516625e1506ae07a73a79f3db35fcf59564dc44cc23700'
    assert sha(BASE/'headless/SIM-005-strain-attribution.md')==sha(BASE/'native/SIM-005-strain-attribution.md')==contract
    frozen='5f8ffcfc077d41a39aa3084466f88806c85a09b4d6ed004213ecfc16d76bbc0e'
    assert sha(BASE/'headless/review-support/expected-peaks-preregistered.json')==sha(BASE/'native/expected-peaks-preregistered.json')==frozen
    binding=read(BASE/'binding/binary-binding.json')
    assert binding['source_commit']==SOURCE and binding['source_files']==37
    assert binding['source_pretest_sha256']==sha(BASE/'binding/source-pretest.json')
    assert binding['acceptance_contract_commit']=='209a8fda72aaca90a08c330427354dd457224a6c' and binding['acceptance_contract_sha256']==contract
    assert binding['independent_expected_peaks_sha256']==frozen and binding['sim_defaults_changed'] is False
    assert binding['package_files']['weftforge']['sha256']=='95fa1b7cb24b8de1a6b507330c2fe43ece0efe5a14226bb28c39f922fcee28c3'
    assert binding['package_files']['weftforge-cli']['sha256']=='c7e63b52d60b293357ba60c49dfed3643472ab177352f2501ebfb207f7ed1092'
    gates=read(BASE/'gates/jobs.json')
    assert [j['name'] for j in gates]==['fmt','tests','clippy','package']
    assert all(j['status']=='completed' and j['exit_code']==0 and j['source_commit']==SOURCE for j in gates)
    assert read(BASE/'gates/current-job.json')=={'status':'completed','all_exit_zero':True,'source_commit':SOURCE}
    count=sum(map(int,re.findall(r'^test result: ok\. (\d+) passed;', (BASE/'gates/tests.log').read_text(),re.MULTILINE)))
    assert count==126
    image_review=read(BASE/'reviews/root-image-review.json')
    assert image_review['source_commit']==SOURCE
    for name,record in image_review['images'].items():
        assert sha(BASE/'native'/name)==record['sha256']
    headless=replay(BASE/'headless/verify.py')
    native=replay(BASE/'native/verify_native_saved.py',BASE/'native',BASE/'headless/baseline-control')
    cli=replay(BASE/'verify_cli_saved.py')
    assert headless['saved_output_replay_passed'] and headless['original_quality_failures_preserved']
    assert headless['meaningful_attribution_frames']==180 and headless['new_simulations_from_verifier']==0
    assert native['status']=='passed' and all(s['original_15_percent_strain_limit_passed'] is False for s in native['stages'])
    assert max(s['maximum_position_difference_m'] for s in native['stages'])==6.938893903907228e-18
    assert cli['status']=='passed' and cli['original_fit_quality']=='failed' and cli['product_commands_executed']==0
    native_task=read(BASE/'binding/native-task-closure.json')
    assert native_task['native_cua_complete'] and native_task['native_pid_left_running']==99058
    assert actual=={p.relative_to(BASE).as_posix() for p in BASE.rglob('*') if p.is_file()},'Replay modified artifact closure'
    print(json.dumps({'kind':'saved_evidence_replay_not_new_publication','status':'passed','candidate_only':args.candidate_only,'git_state':git_state,'artifact_files_including_manifests':len(actual),'source_commit':SOURCE,'source_snapshot_hashes_verified':37,'contract_sha256':contract,'implementation_visibility':'passed','original_garment_quality':'failed','reference_parity':'0/56 coarse contracts; unchanged','headless_public_API_snapshots':181,'headless_meaningful_attributions':180,'headless_prior_positions_velocities_diagnostics':'exact after excluding added attribution and wall time','native_maximum_decimal_export_difference_m':6.938893903907228e-18,'CLI_expected_exits':[1,0,0],'workspace_rust_tests_passed':count,'original_job_status':'all simulation,CLI,build/check workflows terminal; native app intentionally left open at completed Drape','retained_native_app_pid':99058,'actual_binary_scope':'Original executable hashes/process binding are retained provenance; executables are omitted and are not reobserved by saved replay.','new_simulations_product_UI_SDK_network_runs':0,'headless_saved_replay':headless,'native_saved_replay':native,'CLI_saved_replay':cli,'limits':['Source snapshot alone is not a new clean-Git or actual-binary check; default mode additionally verifies current clean/tracked Git/source closure.','Visibility acceptance does not accept the original garment or establish force equilibrium,convergence,calibration,full collision robustness,reference parity or fullSIM005.']},indent=2,allow_nan=False))

if __name__=='__main__':
    main()
