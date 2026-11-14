"""Independent saved-state comparison of the two final preregistered cases."""
from pathlib import Path
import datetime
import hashlib
import json
import math
from balance_measure import add, bends, bending, cross, dynamics, masses, membrane, mul, norm, sewing, sub, dot
from measure import drift, measure, residuals

base=Path(__file__).resolve().parent
h=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
control_meta=json.loads((base/'control/rest-metadata.json').read_text())
control_states=[json.loads(s) for s in (base/'control/trajectory.jsonl').read_text().splitlines()]
control_analysis=json.loads((base/'control-analysis.json').read_text())
registration=json.loads((base/'plan-probes-001.json').read_text())
assert h(base/'target/release/cloth-balance-probe')==registration['executed_probe_binary_sha256']
assert h(base/'original-grain-shirt.weftforge')==registration['fixture_sha256']
all_cases={}
all_forces=[]
for name,frames in [('zero-g-balance',180),('no-avatar',60)]:
    ledger=json.loads((base/(name+'-job.json')).read_text())
    assert ledger['exit_code']==0
    meta=json.loads((base/name/'rest-metadata.json').read_text())
    states=[json.loads(s) for s in (base/name/'trajectory.jsonl').read_text().splitlines()]
    assert len(states)==frames+1
    assert [s['global_frame'] for s in states]==list(range(frames+1))
    assert all(s['settings_last_step']['gravity']==[0.,0.,0.] for s in states)
    assert meta['original_project']==control_meta['project']
    expected=json.loads(json.dumps(control_meta['project']))
    if name=='no-avatar':
        expected['mannequin']['capsules']=[]
    assert meta['project']==expected
    mesh_keys=['rest_positions_m','uv_m','triangles','piece_ids','stitches','masses_kg','boundary']
    assert all(meta[k]==control_meta[k] for k in mesh_keys)
    physical_mass=masses(meta)
    assert max(abs(a-b) for a,b in zip(physical_mass,meta['masses_kg']))<1e-14
    prefix=None
    if name=='zero-g-balance':
        prefix={'frames':61,'positions_and_velocities_exact':all(s['positions_m']==c['positions_m'] and s['velocities_m_per_s']==c['velocities_m_per_s'] for s,c in zip(states[:61],control_states[:61]))}
        assert prefix['positions_and_velocities_exact']
    constraints=bends(meta)
    samples=[]
    previous=None
    for state in states:
        frame=state['global_frame'];pos=state['positions_m'];vel=state['velocities_m_per_s']
        strain=measure(meta,pos);supported=residuals(meta,pos);motion=dynamics(meta,pos,vel)
        assert abs(strain['peak']['absolute_strain']-state['diagnostics']['max_stretch_error'])<1e-12
        assert abs(motion['kinetic_energy_J']-state['diagnostics']['kinetic_energy'])<1e-12
        em,fm=membrane(meta,pos);es,fs=sewing(meta,pos,frame/60)
        compute_force=frame in [60,frames]
        eb,fb,degenerate=bending(constraints,pos,compute_force)
        peak=strain['peak']
        peak['uv_m']=[meta['uv_m'][v] for v in peak['vertices_0based']]
        sample={'frame':frame,'time_s':frame/60,'strain':strain,'motion':motion,'independent_supported_residuals':supported,'implementation_diagnostics':state['diagnostics'],'bending_energy_J':eb,'bending_degenerate_count':degenerate,'temporary_sewing_energy_J':es,'membrane_bending_energy_J':em+eb}
        if previous is not None:
            sample['last_frame_effective_total_net_force_N']=mul(sub(motion['momentum_kg_m_per_s'],previous['motion']['momentum_kg_m_per_s']),60.)
            acceleration=[mul(sub(a,b),60.) for a,b in zip(vel,states[frame-1]['velocities_m_per_s'])]
            sample['last_frame_nodal_effective_net_force_max_N']=max(m*norm(a) for m,a in zip(physical_mass,acceleration))
            sample['last_frame_nodal_acceleration_mass_rms_m_per_s2']=math.sqrt(sum(m*dot(a,a) for m,a in zip(physical_mass,acceleration))/sum(physical_mass))
            sample['last_frame_nodal_drift_m']=drift(pos,states[frame-1]['positions_m'],physical_mass)
        if compute_force:
            force=[add(add(a,b),c) for a,b,c in zip(fm,fb,fs)]
            sample['independent_conservative_force_summary']={'membrane_max_N':max(map(norm,fm)),'bending_max_N':max(map(norm,fb)),'sewing_max_N':max(map(norm,fs)),'combined_max_N':max(map(norm,force)),'internal_resultant_N':[sum(f[k] for f in force) for k in range(3)],'excludes':'Contact/friction reactions and damping; not complete force balance'}
            all_forces.append({'case':name,'frame':frame,'membrane_force_N':fm,'bending_force_N':fb,'sewing_force_N':fs,'combined_conservative_force_N':force})
        sample['arithmetic_screen_passed']=state['diagnostics']['finite'] and state['diagnostics']['seam_assembly_remaining']==0 and peak['absolute_strain']<.15 and supported['sampled_stitch_gap_m']<.005 and state['diagnostics']['max_contact_error']<.002
        sample['original_garment_quality_passed']=sample['arithmetic_screen_passed'] if name!='no-avatar' else None
        samples.append(sample);previous=sample
    at1=samples[60]
    terminal=samples[-1]
    control_at_time=control_analysis['samples'][frames]
    comparison={'t1_peak_strain':at1['strain']['peak']['absolute_strain'],'terminal_peak_strain':terminal['strain']['peak']['absolute_strain'],'terminal_strain_difference_from_original_same_time_control':terminal['strain']['peak']['absolute_strain']-control_at_time['strain']['peak']['absolute_strain'],'terminal_nodal_drift_from_original_same_time_control_m':drift(states[-1]['positions_m'],control_states[frames]['positions_m'],physical_mass),'terminal_velocity_drift_from_original_same_time_control_m_per_s':drift(states[-1]['velocities_m_per_s'],control_states[frames]['velocities_m_per_s'],physical_mass),'terminal_minus_t1_peak_strain':terminal['strain']['peak']['absolute_strain']-at1['strain']['peak']['absolute_strain'],'terminal_nodal_drift_from_own_t1_m':drift(states[-1]['positions_m'],states[60]['positions_m'],physical_mass),'terminal_elastic_energy_J':terminal['membrane_bending_energy_J'],'terminal_seam_m':terminal['independent_supported_residuals']['sampled_stitch_gap_m'],'terminal_supported_contact_m':terminal['implementation_diagnostics']['max_contact_error'],'terminal_rms_speed_m_per_s':terminal['motion']['rms_speed_m_per_s'],'terminal_KE_J':terminal['motion']['kinetic_energy_J'],'terminal_original_quality_passed':terminal['original_garment_quality_passed']}
    comparison['strain_sensitivity_trigger_exceeded']=abs(comparison['terminal_strain_difference_from_original_same_time_control'])>.005
    comparison['nodal_sensitivity_trigger_exceeded']=comparison['terminal_nodal_drift_from_original_same_time_control_m']['max_m']>.001
    if name=='zero-g-balance':
        last=samples[151:181]
        comparison['last_half_second_peak_strain_min_max']=[min(s['strain']['peak']['absolute_strain'] for s in last),max(s['strain']['peak']['absolute_strain'] for s in last)]
        comparison['last_half_second_nodal_drift_m']=drift(states[180]['positions_m'],states[150]['positions_m'],physical_mass)
        comparison['last_half_second_elastic_energy_change_J']=samples[180]['membrane_bending_energy_J']-samples[150]['membrane_bending_energy_J']
    all_cases[name]={'prefix_check':prefix,'original_fixture_unchanged':True,'mesh_exactly_matches_control':True,'runtime_project_change':'none' if name=='zero-g-balance' else 'only mannequin.capsules cleared in memory; original project retained','comparison':comparison,'samples':samples,'job':{k:ledger[k] for k in ['exit_code','started_at','completed_at','wall_seconds','launcher_pid','child_pid','child_process_group']}}
report={'kind':'private_implementation_only_three_case_cloth_causal_diagnosis','analyzed_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'source_commit':registration['source_commit'],'plan_probes_sha256':h(base/'plan-probes-001.json'),'probe_binary_sha256':h(base/'target/release/cloth-balance-probe'),'fixture_sha256':h(base/'original-grain-shirt.weftforge'),'cases':all_cases,'limits':['Only three preregistered original engineering cases; no parameter sweep or threshold changes','Avatar removal is a counterfactual diagnostic and cannot count as garment quality','Low KE and small frame changes do not establish full force equilibrium','No contact-reaction/multiplier reconstruction; force equilibrium and solver defect remain unknown','No convergence, calibrated fabric or reference application result']}
(base/'probes-analysis.json').write_text(json.dumps(report,indent=2)+'\n')
(base/'probes-independent-forces.json').write_text(json.dumps(all_forces,indent=2)+'\n')
for name,result in all_cases.items():
    print(name,json.dumps(result['comparison'],indent=2))
