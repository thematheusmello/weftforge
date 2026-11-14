"""Saved-state analysis; no simulator calls and no fixture edits."""
from pathlib import Path
import datetime
import hashlib
import json
import math
import subprocess
from balance_measure import add, angle, bends, bending, cross, dynamics, masses, membrane, mul, norm, sewing, sub, dot, weighted
from measure import drift, measure, residuals

base=Path(__file__).resolve().parent
repo=base.parents[1]
meta=json.loads((base/'control/rest-metadata.json').read_text())
states=[json.loads(row) for row in (base/'control/trajectory.jsonl').read_text().splitlines()]
assert len(states)==181 and [s['global_frame'] for s in states]==list(range(181))
assert json.loads((base/'control-job.json').read_text())['exit_code']==0
hashfile=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
source_manifest=json.loads((base/'source-manifest.json').read_text())
assert all(hashfile(base/'source'/p)==h for p,h in source_manifest['files'].items())
assert subprocess.check_output(['git','-C',str(base/'source'),'status','--porcelain'],text=True)==''
assert hashfile(base/'original-grain-shirt.weftforge')=='82e7c6d398c7772d7e62c28196dcf11845020020d44fca9ac9113e615b9537e7'
physical_mass=masses(meta)
assert max(abs(a-b) for a,b in zip(physical_mass,meta['masses_kg']))<1e-14
assert meta['rest_positions_m']==states[0]['positions_m']
assert all(v==0 for q in states[0]['velocities_m_per_s'] for v in q)
constraints=bends(meta)
seam_nodes=set(v for s in meta['stitches'] for end in [s['a'],s['b']] for v in end['vertices'])
node_curves={}
for sample in meta['boundary']:
    node_curves.setdefault(sample['vertex'],set()).add(sample['curve'])
samples=[]
forces=[]
previous=None
for state in states:
    frame=state['global_frame']
    pos=state['positions_m']
    velocity=state['velocities_m_per_s']
    strain=measure(meta,pos)
    supported=residuals(meta,pos)
    motion=dynamics(meta,pos,velocity)
    assert abs(strain['peak']['absolute_strain']-state['diagnostics']['max_stretch_error'])<1e-12
    assert abs(motion['kinetic_energy_J']-state['diagnostics']['kinetic_energy'])<1e-12
    em,fm=membrane(meta,pos)
    assert abs(em-strain['membrane_energy_J']['total'])<1e-12
    es,fs=sewing(meta,pos,frame/60)
    compute_force=frame in [0,30,45,60,180]
    eb,fb,degen=bending(constraints,pos,compute_force)
    peak=strain['peak']
    peak['uv_m']=[meta['uv_m'][v] for v in peak['vertices_0based']]
    peak['positions_m']=[pos[v] for v in peak['vertices_0based']]
    peak['source_boundary_curves']={str(v):sorted(node_curves.get(v,[])) for v in peak['vertices_0based']}
    peak['vertices_in_stitch_support']=[v for v in peak['vertices_0based'] if v in seam_nodes]
    record={'frame':frame,'time_s':frame/60,'phase':state['phase'],'strain':strain,'independent_supported_residuals':supported,'implementation_diagnostics':state['diagnostics'],'motion':motion,'bending_energy_J':eb,'bending_degenerate_count':degen,'temporary_sewing_energy_J':es,'membrane_bending_energy_J':em+eb}
    if previous is not None:
        prev_motion=previous['motion']
        record['last_frame_effective_total_net_force_N']=mul(sub(motion['momentum_kg_m_per_s'],prev_motion['momentum_kg_m_per_s']),60.)
        acceleration=[mul(sub(v0,v1),60.) for v0,v1 in zip(velocity,states[frame-1]['velocities_m_per_s'])]
        record['last_frame_nodal_effective_net_force_max_N']=max(m*norm(a) for m,a in zip(physical_mass,acceleration))
        record['last_frame_nodal_acceleration_mass_rms_m_per_s2']=math.sqrt(sum(m*dot(a,a) for m,a in zip(physical_mass,acceleration))/sum(physical_mass))
        record['last_frame_nodal_drift_m']=drift(pos,states[frame-1]['positions_m'],physical_mass)
        record['last_frame_membrane_bending_energy_change_J']=em+eb-previous['membrane_bending_energy_J']
    if compute_force:
        all_force=[add(add(a,b),c) for a,b,c in zip(fm,fb,fs)]
        elastic=[add(a,b) for a,b in zip(fm,fb)]
        resultant=[sum(f[k] for f in all_force) for k in range(3)]
        torque=[0.,0.,0.]
        for x,f in zip(pos,all_force):
            torque=add(torque,cross(sub(x,motion['center_of_mass_m']),f))
        record['independent_conservative_force_summary']={'membrane_max_N':max(map(norm,fm)),'bending_max_N':max(map(norm,fb)),'sewing_max_N':max(map(norm,fs)),'combined_max_N':max(map(norm,all_force)),'combined_internal_resultant_N':resultant,'combined_internal_torque_N_m':torque,'membrane_bending_force_rms_N':math.sqrt(sum(dot(f,f) for f in elastic)/len(elastic)),'excludes':'Contact/friction reactions and damping; not a complete equilibrium residual','bending_force_finite_difference_epsilon_m':1e-6}
        forces.append({'frame':frame,'membrane_force_N':fm,'bending_force_N':fb,'sewing_force_N':fs,'combined_conservative_force_N':all_force})
    record['quality_passed']=state['diagnostics']['finite'] and state['diagnostics']['seam_assembly_remaining']==0 and peak['absolute_strain']<.15 and supported['sampled_stitch_gap_m']<.005 and state['diagnostics']['max_contact_error']<.002
    samples.append(record)
    previous=record
historical_dir=repo/'parity/reports/artifacts/physics-full-path-20261114'
historical_meta=json.loads((historical_dir/'runs/rest-metadata.json').read_text())
mesh_keys=['rest_positions_m','uv_m','triangles','piece_ids','masses_kg']
mesh_equal={k:meta[k]==historical_meta[k] for k in mesh_keys}
mesh_equal['stitches']=all({k:s[k] for k in ['seam','a','b']}==old for s,old in zip(meta['stitches'],historical_meta['stitches'])) and len(meta['stitches'])==len(historical_meta['stitches'])
historical_states=[json.loads(s) for s in (historical_dir/'runs/trajectory.jsonl').read_text().splitlines()]
history=[]
for old in historical_states:
    if old['case']!='control_8x32':
        continue
    new=states[old['global_frame']]
    row={'frame':old['global_frame'],'historical_source':'e342471b18bc5d636d8e869ba0d374710ea15da3','current_source':source_manifest['commit'],'historical_independent_peak_axis_strain':measure(historical_meta,old['positions_m'])['peak']['absolute_strain'],'current_independent_peak_axis_strain':samples[old['global_frame']]['strain']['peak']['absolute_strain']}
    row['axis_strain_difference']=row['current_independent_peak_axis_strain']-row['historical_independent_peak_axis_strain']
    if all(mesh_equal.values()):
        row['nodal_difference_m']=drift(new['positions_m'],old['positions_m'],physical_mass)
        row['velocity_difference_m_per_s']=drift(new['velocities_m_per_s'],old['velocities_m_per_s'],physical_mass)
    else:
        row['nodal_comparison']='not attempted; identity/topology differs'
    history.append(row)
project=meta['project']
points={p['id']:p['position'] for p in project['points']}
curves={c['id']:c for c in project['curves']}
def length(cid):
    c=curves[cid]
    assert c['controls'] is None
    return norm(sub(points[c['end']],points[c['start']]))
seam_mapping=[]
for seam in project['seams']:
    a,b=curves[seam['a']['curve']],curves[seam['b']['curve']]
    matching=[s for s in meta['stitches'] if s['seam']==seam['id']]
    offsets=[sub(weighted(s['a'],meta['rest_positions_m']),weighted(s['b'],meta['rest_positions_m'])) for s in matching]
    seam_mapping.append({'seam':seam['id'],'a_curve':a['id'],'b_curve':b['id'],'side_lengths_m':[length(a['id']),length(b['id'])],'length_difference_m':length(a['id'])-length(b['id']),'source_correspondence':seam['correspondence'],'weighted_stitch_count':len(matching),'max_arranged_xy_mismatch_m':max(math.hypot(d[0],d[1]) for d in offsets),'arranged_z_gaps_m':sorted(set(d[2] for d in offsets))})
loops=[]
for label,ids,radius in [('hem_body',[17,46],.183),('neck',[23,24,25,52,53,54],.108),('right_cuff',[20,49],.068),('left_cuff',[28,57],.068)]:
    rest=sum(map(length,ids));circ=2*math.pi*radius
    loops.append({'opening':label,'source_curves':ids,'rest_perimeter_m':rest,'conditional_inflated_cylinder_circumference_m':circ,'conditional_uniform_tangent_extension_required':circ/rest-1,'applicability':'Only a closed loop enclosing and constrained outside this cylinder at the corresponding axial section; finite capsules and migration/diagonal drape can invalidate the section assumption. This is not a proof that the original 15% material-axis limit is impossible.'})
mesh_angles=[]
for tri in meta['triangles']:
    p=[meta['uv_m'][v] for v in tri]
    for k in range(3):
        a,b=sub(p[(k+1)%3],p[k]),sub(p[(k+2)%3],p[k])
        mesh_angles.append(math.degrees(math.acos(max(-1.,min(1.,dot(a,b)/(norm(a)*norm(b)))))))
first_high=next((s['frame'] for s in samples if s['strain']['peak']['absolute_strain']>=.15),None)
first_closed=next((s['frame'] for s in samples if s['implementation_diagnostics']['seam_assembly_remaining']==0),None)
first_gap_pass=next((s['frame'] for s in samples if s['independent_supported_residuals']['sampled_stitch_gap_m']<.005),None)
result={'kind':'private_implementation_only_saved_state_cloth_diagnosis','analyzed_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'source_commit':source_manifest['commit'],'source_manifest_sha256':hashfile(base/'source-manifest.json'),'plan_sha256':hashfile(base/'plan.json'),'plan_erratum_sha256':hashfile(base/'plan-erratum-001.json'),'binary_sha256':hashfile(base/'target/release/cloth-balance-diagnostic'),'fixture_sha256':hashfile(base/'original-grain-shirt.weftforge'),'mesh':{'vertices':len(meta['uv_m']),'triangles':len(meta['triangles']),'stitches':len(meta['stitches']),'bends':len(constraints),'min_material_angle_degrees':min(mesh_angles),'max_material_angle_degrees':max(mesh_angles)},'independent_mass_max_difference_kg':max(abs(a-b) for a,b in zip(physical_mass,meta['masses_kg'])),'historical_mesh_exact_equality':mesh_equal,'first_frame_axis_strain_at_least_15percent':first_high,'first_frame_assembly_target_zero':first_closed,'first_frame_sampled_seam_gap_below_5mm':first_gap_pass,'seam_mapping':seam_mapping,'conditional_opening_bounds':loops,'samples':samples,'historical_shared_time_comparison':history,'conclusions':{'initial_arrangement_strain':samples[0]['strain']['peak']['absolute_strain'],'assembly_quality_passed':samples[60]['quality_passed'],'final_quality_passed':samples[180]['quality_passed'],'force_equilibrium':'unknown; contact/friction reactions and complete self-contact force mapping unavailable','limits':['Finite interval momentum/velocity changes do not establish force equilibrium','Conservative forces are geometric energy gradients; sequential XPBD effective impulses need not equal final elastic gradient at finite convergence','Conditional opening circumference estimates do not prove impossibility under the axis strain screen, which also leaves shear unconstrained','No reference application or calibrated fabric result','No production/source/default/fixture/threshold changes']}}
(base/'control-analysis.json').write_text(json.dumps(result,indent=2)+'\n')
(base/'control-independent-forces.json').write_text(json.dumps(forces,indent=2)+'\n')
print(json.dumps({k:v for k,v in result.items() if k not in ['samples','historical_shared_time_comparison','seam_mapping','conditional_opening_bounds']},indent=2))
for frame in [0,15,27,30,45,60,180]:
    s=samples[frame]
    print(json.dumps({'frame':frame,'peak':s['strain']['peak'],'motion':s['motion'],'independent_residuals':s['independent_supported_residuals'],'membrane_bending_energy_J':s['membrane_bending_energy_J'],'conservative_force_summary':s.get('independent_conservative_force_summary')}))
