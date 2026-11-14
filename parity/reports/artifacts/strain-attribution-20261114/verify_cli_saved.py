"""Check the three retained CLI receipts without executing any product command."""
import sys
sys.dont_write_bytecode = True
import hashlib
import importlib.util
import json
from pathlib import Path

BASE=Path(__file__).resolve().parent
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
read=lambda p:json.loads(p.read_bytes())

def module(name,path):
    spec=importlib.util.spec_from_file_location(name,path)
    result=importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result

def main():
    directory=BASE/'cli'
    plan=read(directory/'plan.json')
    assert plan['source_commit']=='6c45535cb312f7fed8ad872ee87b6ba5e5974e35'
    assert sha(directory/'check_cli.py')==plan['checker_sha256']=='6bba7c30c505db8ca2d0da10c7bfd37f8e9ea7a81d0b71a97b53c8c8e66324f4'
    expected_path=BASE/'headless/review-support/expected-peaks-preregistered.json'
    assert sha(expected_path)==plan['expected_peaks_sha256']=='5f8ffcfc077d41a39aa3084466f88806c85a09b4d6ed004213ecfc16d76bbc0e'
    original_hash='82e7c6d398c7772d7e62c28196dcf11845020020d44fca9ac9113e615b9537e7'
    assert sha(directory/'original.weftforge')==sha(BASE/'headless/original-grain-shirt.weftforge')==plan['input_sha256']==original_hash
    assert (directory/'retained-output.obj').read_bytes()==b'previous independently retained garment output\n'
    assert plan['commands']==[['fit','original.weftforge','retained-output.obj'],['simulate','original.weftforge','one-step.obj','1'],['benchmark','original.weftforge','1']]
    jobs=read(directory/'jobs.json')
    assert len(jobs)==3 and read(directory/'current-job.json')=={'status':'completed','all_expected_exits':True}
    output={}
    for job,command,code in zip(jobs,plan['commands'],[1,0,0]):
        assert job['command'][1:]==command and job['exit_code']==code
        assert job['command'][0]=='<workspace>/.local/candidates/6c45535cb312/weftforge-cli'
        assert plan['frozen_at']<job['started_at']<job['completed_at']
        output[command[0]]=read(directory/(command[0]+'.stdout'))
    assert (directory/'fit.stderr').read_text()=='weftforge-cli: fitting quality check failed; see JSON report; prior output retained\n'
    assert not (directory/'simulate.stderr').read_bytes() and not (directory/'benchmark.stderr').read_bytes()
    fit=output['fit']
    assert fit['kind']=='implementation_only' and fit['status']=='failed' and fit['export_written'] is False and fit['solver_error'] is None
    assert fit['criteria']=={'finite':True,'max_contact_penetration_below_m':.002,'max_seam_gap_below_m':.005,'max_stretch_error_below':.15,'seam_assembly_remaining_m':0.}
    expected={r['frame']:r for r in read(expected_path)['rows'] if r['case']=='control'}
    meta=read(BASE/'headless/baseline-control/rest-metadata.json')
    states={r['global_frame']:r for r in map(json.loads,(BASE/'headless/control/trajectory.jsonl').read_text().splitlines())}
    pieces={p['id']:p for p in meta['project']['pieces']}
    for phase,frame,piece_id in [(fit['phases'][0],60,31),(fit['phases'][1],180,2),(fit,180,2)]:
        peak=phase['diagnostics']['peak_strain']
        oracle=expected[frame]
        assert peak['triangle']==oracle['triangle'] and peak['axis']==oracle['axis']=='warp'
        assert abs(peak['signed_strain']-oracle['signed_strain'])<1e-12
        assert phase['peak_strain_piece']=={'id':piece_id,'name':pieces[piece_id]['name']}
        observed={k:v for k,v in phase['diagnostics'].items() if k!='step_seconds'}
        retained={k:v for k,v in states[frame]['diagnostics'].items() if k!='step_seconds'}
        assert observed==retained
        assert phase['diagnostics']['max_stretch_error']>.15
    for phase,name,frames,gravity in zip(fit['phases'],['assembly','drape'],[60,120],[[0.,0.,0.],[0.,-9.81,0.]]):
        assert phase['name']==name and phase['steps_requested']==phase['steps_completed']==frames
        assert phase['settings']=={'dt':1/60,'gravity':gravity,'iterations':32,'seam_closure_speed':1.,'substeps':8}
    reviewer=module('frozen_independent_material_review',BASE/'headless/review-support/replay_review.py')
    assert sha(BASE/'headless/review-support/replay_review.py')=='ef19fef14b25088d951823c11da0b9f8e1733dd8c8137353e66186c0eb3eabdb'
    reviewer.self_check()
    native_parser=module('frozen_native_obj_checker',BASE/'native/verify_native_saved.py')
    positions,uv,triangles,groups,_,uv_text=native_parser.parse_obj((directory/'one-step.obj').read_text())
    assert triangles==meta['triangles'] and len(positions)==len(meta['rest_positions_m'])==150
    assert groups==[meta['piece_ids'][t[0]] for t in triangles]
    assert uv_text==[[f'{value:.17f}' for value in row] for row in meta['uv_m']]
    independent=reviewer.membrane(reviewer.triangle_data(meta)[0],positions)[0]['peak']
    for name in ['simulate','benchmark']:
        receipt=output[name];peak=receipt['diagnostics']['peak_strain']
        assert receipt['kind']=='implementation_only' and receipt['steps']==1
        assert peak['triangle']==independent['triangle'] and peak['axis']==independent['axis']
        assert abs(peak['signed_strain']-independent['signed_strain'])<1e-12
        assert abs(receipt['diagnostics']['max_stretch_error']-independent['absolute_strain'])<1e-12
        assert receipt['peak_strain_piece']=={'id':independent['piece'],'name':pieces[independent['piece']]['name']}
        assert receipt['diagnostics']['finite'] and receipt['settings']=={'dt':1/60,'gravity':[0.,-9.81,0.],'iterations':8,'seam_closure_speed':.5,'substeps':4}
    summary=read(directory/'summary.json')
    assert summary['status']=='passed' and summary['jobs']==jobs and summary['source_unchanged'] and summary['prior_output_unchanged']
    assert summary['original_fit_quality']=='failed' and summary['independent_one_step_peak']==independent
    assert summary['fit_assembly_peak']==fit['phases'][0]['diagnostics']['peak_strain'] and summary['fit_drape_peak']==fit['diagnostics']['peak_strain']
    assert plan['binary_sha256']==read(BASE/'binding/binary-binding.json')['package_files']['weftforge-cli']['sha256']=='c7e63b52d60b293357ba60c49dfed3643472ab177352f2501ebfb207f7ed1092'
    print(json.dumps({'kind':'preparation_time_saved_CLI_replay_not_new_publication','status':'passed','product_commands_executed':0,'three_expected_exit_codes':[1,0,0],'source_input_unchanged':True,'prior_export_sentinel_unchanged':True,'original_fit_quality':'failed','fit_assembly_piece_id':31,'fit_drape_piece_id':2,'independent_one_step_peak':independent,'binary_scope':'retained original actual-executable binding; binary omitted from public candidate'},indent=2,allow_nan=False))

if __name__=='__main__':
    main()
