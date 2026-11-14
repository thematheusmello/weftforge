"""Frozen POST-FAILURE reader: qualify retained prefixes, never overall physical quality."""
from pathlib import Path
import datetime
import hashlib
import json
import math
import sys
import struct
import metrics

ROOT=Path(sys.argv[1]).resolve()
OUTPUT=Path(sys.argv[2]).resolve()
NATIVE=Path(sys.argv[3]).resolve()
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()


def load(path):
    return json.loads(path.read_text(),parse_constant=lambda value: (_ for _ in ()).throw(ValueError(f"Non-finite JSON token: {value}")))


def lines(path):
    return [json.loads(line) for line in path.read_text().splitlines()]


def save(path, value):
    with path.open("x") as out:
        json.dump(value,out,indent=2,allow_nan=False);out.write("\n")


def exact(a,b):
    if isinstance(a,(int,float)) and not isinstance(a,bool) and isinstance(b,(int,float)) and not isinstance(b,bool):
        return struct.pack(">d",float(a))==struct.pack(">d",float(b))
    if isinstance(a,list) and isinstance(b,list):
        return len(a)==len(b) and all(exact(x,y) for x,y in zip(a,b))
    if isinstance(a,dict) and isinstance(b,dict):
        return a.keys()==b.keys() and all(exact(a[k],b[k]) for k in a)
    return type(a) is type(b) and a==b


def exact_rest(actual,expected):
    a=dict(actual)
    a.pop("masses_kg")
    assert exact(a,expected),"Actual solver rest metadata differs from the frozen geometry-only snapshot"


def assert_close(a,b,tolerance,label):
    assert math.isfinite(a) and math.isfinite(b) and abs(a-b)<=tolerance,(label,a,b,tolerance)


def quality(strain,seam,contact,remaining,finite=True):
    return {"finite":finite,"completed_assembly":remaining==0.,"material_axis_strain_below_0_15":strain<.15,"sampled_seam_gap_below_0_005_m":seam<.005,"supported_contact_penetration_below_0_002_m":contact<.002}


def analyze_case(name):
    assert name in ("base","height")
    run=ROOT/"runs"/name
    result={"kind":"POST-FAILURE read-only prefix qualification; not prospective acceptance","case":name,"status":"incomplete_prefix_only","arithmetic_consistency":"failed","reference_parity":"not measured","equilibrium_claim":False,"overall_quality":"not_qualified_incomplete_prefix_only","rows":[]}
    try:
        meta=load(run/"rest-metadata.json")
        exact_rest(meta,load(ROOT/f"prepared/{name}/mesh-metadata.json"))
        expected_masses=metrics.masses(meta)
        assert len(expected_masses)==len(meta["masses_kg"])
        mass_error=max(abs(a-b) for a,b in zip(expected_masses,meta["masses_kg"]))
        assert mass_error<=1e-12
        result["mass_check"]={"max_nodal_error_kg":mass_error,"independent_total_kg":sum(expected_masses),"solver_total_kg":sum(meta["masses_kg"])}
        states,steps=lines(run/"trajectory.jsonl"),lines(run/"steps.jsonl")
        assert len(states)=={"base":164,"height":159}[name]
        assert states and len(states)==len(steps)+1
        assert states[0]["positions_m"]==meta["rest_positions_m"]
        assert all(q==0. for v in states[0]["velocities_m_per_s"] for q in v)
        excluded=metrics.exclusion(meta)
        def endpoint(s,key):
            e=s[key]
            return metrics.add(metrics.mul(meta["rest_positions_m"][e["vertices"][0]],1-e["weight"]),metrics.mul(meta["rest_positions_m"][e["vertices"][1]],e["weight"]))
        targets=[metrics.norm(metrics.sub(endpoint(s,"a"),endpoint(s,"b"))) for s in meta["stitches"]]
        for frame,state in enumerate(states):
            assert state["global_frame"]==frame
            settings={"dt":1/60,"substeps":8,"iterations":32,"seam_closure_speed":1.,"gravity":[0.,0.,0.] if frame<=60 else [0.,-9.81,0.]}
            assert state["settings_last_step"]==settings
            assert state["phase"]==("assembly" if frame<=60 else "drape")
            if frame:
                for _ in range(8):
                    targets=[max(0.,t-1/480) for t in targets]
                assert steps[frame-1]["global_frame"]==frame and steps[frame-1]["settings"]==settings
                assert exact(steps[frame-1]["diagnostics"],state["diagnostics"])
            x,v=state["positions_m"],state["velocities_m_per_s"]
            assert len(x)==len(v)==len(expected_masses)
            finite=all(len(p)==3 and all(isinstance(q,(int,float)) and math.isfinite(q) for q in p) for p in x+v)
            assert finite,"Non-finite/invalid recorded state retained in the numerical output"
            d=state["diagnostics"]
            strain=metrics.measure(meta,x)
            residuals=metrics.residuals(meta,x)
            analytic=metrics.analytic_contacts(meta,x)
            contact=metrics.supported_self_contact(meta,x,excluded)
            maximum_contact=max(analytic["avatar_ground_penetration_m"],contact["supported_static_self_interpanel_penetration_m"])
            kinetic=sum(m*metrics.dot(velocity,velocity)/2 for m,velocity in zip(expected_masses,v))
            assert_close(d["max_stretch_error"],strain["peak"]["absolute_strain"],1e-12,"material-axis strain")
            assert_close(d["max_seam_error"],residuals["sampled_stitch_gap_m"],1e-12,"weighted stitch gap")
            assert_close(d["max_contact_error"],maximum_contact,1e-10,"analytic plus supported static contact")
            assert_close(d["seam_assembly_remaining"],max(targets,default=0.),1e-12,"independent assembly recurrence")
            assert_close(d["kinetic_energy"],kinetic,1e-12*(1+abs(kinetic)),"mass-based kinetic energy")
            assert_close(d["simulation_time"],frame/60,1e-12,"simulation time")
            peak=d["peak_strain"]
            axis_strains=metrics.signed_triangle_axes(meta,x,peak["triangle"])
            assert_close(peak["signed_strain"],axis_strains[peak["axis"]],1e-12,"reported peak triangle/axis signed strain")
            assert_close(abs(peak["signed_strain"]),strain["peak"]["absolute_strain"],1e-12,"reported peak magnitude")
            if frame==0:
                assert d["initial_intersections"]==analytic["intersection_pairs_depth_over_1e_minus8"]+contact["intersection_pairs_depth_over_1e_minus8"]
            gates=quality(strain["peak"]["absolute_strain"],residuals["sampled_stitch_gap_m"],maximum_contact,max(targets,default=0.),finite and d["finite"])
            row={"frame":frame,"diagnostics":d,"independent_material_axis":strain,"independent_seam":residuals["sampled_stitch_gap_m"],"independent_analytic_contact":analytic,"independent_supported_static_contact":contact,"observed_prefix_threshold_checks_not_overall_quality":gates,"rms_speed_m_per_s":math.sqrt(2*kinetic/sum(expected_masses)),"max_speed_m_per_s":max(map(metrics.norm,v))}
            if frame==60:
                row["obj_check"]=metrics.check_obj(meta,x,run/f"stage-{frame:03}.obj")
            result["rows"].append(row)
            if frame%15==0:
                print(json.dumps({"case":name,"accepted_prefix_frame_checked":frame,"status":"incomplete_prefix_only"}),flush=True)
        failed=load(run/"failed-step.json")
        assert failed["attempted_frame"]==len(states) and failed["last_retained_frame"]==len(states)-1
        assert "cancelled" in failed["error"]
        for key in ("positions_m","velocities_m_per_s","diagnostics","global_frame","phase","settings_last_step"):
            assert exact(states[-1][key],failed["retained_state"][key]),key
        assert not (run/"completed-case.json").exists() and not (run/"stage-180.obj").exists()
        result.update(arithmetic_consistency="verified_on_all_available_states",accepted_prefix_frames=len(steps),boundary_states_checked=len(states),cancellation_retention="last accepted positions/velocities/diagnostics/frame/phase/settings exact",all_original_contact_enabled=True,full181_state_gate="failed_incomplete",completed180_frame_gate="failed_incomplete",stage180_gate="missing",full_native_0_60_180_gate="not_run_incomplete",cross_case_final_curve_t_gate="not_run_incomplete")
    except Exception as exc:
        result["first_failure"]=f"{type(exc).__name__}: {exc}"
    result["completed_utc"]=datetime.datetime.now(datetime.timezone.utc).isoformat()
    save(OUTPUT/f"{name}-prefix.json",result)
    print(json.dumps({key:result.get(key) for key in ("case","status","arithmetic_consistency","boundary_states_checked","first_failure")}),flush=True)
    return 0 if result["arithmetic_consistency"]=="verified_on_all_available_states" else 1


def native_available():
    receipt=load(NATIVE/"case-receipt.json")
    expected=sha(ROOT/"inputs/base.weftforge")
    assert receipt["input_sha256"]==receipt["reference_input_sha256"]==expected
    assert receipt["actual_accepted_checkpoints"]==[0,60] and receipt["missing_checkpoints"]==[180]
    assert receipt["dt_s"]==1/60 and receipt["substeps"]==8 and receipt["iterations"]==32 and receipt["seam_closure_m_per_s"]==1.
    assert receipt["native_runtime_sha256"]=="95fa1b7cb24b8de1a6b507330c2fe43ece0efe5a14226bb28c39f922fcee28c3"
    for filename in ("stage-000.weftforge","stage-060.weftforge","cap-retained.weftforge"):
        assert sha(NATIVE/filename)==expected
    meta=load(ROOT/"runs/base/rest-metadata.json")
    states=lines(ROOT/"runs/base/trajectory.jsonl")
    rows=[]
    for frame in (0,60):
        native=NATIVE/f"stage-{frame:03}.obj"
        headless=ROOT/("prepared/base/rest.obj" if frame==0 else "runs/base/stage-060.obj")
        check=metrics.check_obj(meta,states[frame]["positions_m"],native)
        assert native.read_bytes()==headless.read_bytes()
        rows.append({"frame":frame,"simulation_time_s":frame/60,"native_OBJ_sha256":sha(native),"headless_OBJ_sha256":sha(headless),"entire_OBJ_bytes_identical":True,"frozen_decimal_topology_normals_check":check})
    assert (NATIVE/"cap-retained.obj").read_bytes()==(NATIVE/"stage-060.obj").read_bytes()
    assert not (NATIVE/"stage-180.obj").exists()
    jpeg_records=load(Path(__file__).resolve().parent/"inputs/native-jpeg-records.json")
    events=lines(Path(__file__).resolve().parent/"inputs/native-events.jsonl")
    for row in jpeg_records["records"]:
        assert sha(NATIVE/row["source_filename"])==row["sha256"]
        matching=[event for event in events if event.get("action")=="Native stage observation" and event.get("case")=="base" and event.get("stem")==row["source_filename"][:-4]]
        assert len(matching)==1,(row["source_filename"],len(matching))
        event=matching[0]
        assert event["capture_started"]["utc"]==row["capture_started_utc"] and event["utc"]==row["capture_recorded_utc"]
    result={"kind":"POST-FAILURE read-only available native checkpoint qualification","status":"incomplete_prefix_only","native_receipt_sha256":sha(NATIVE/"case-receipt.json"),"source_input_sha256":expected,"accepted_checkpoints":rows,"source_JPEG_records":jpeg_records,"native_A_180":"missing","native_B":"not_run_missing_accepted_A_post_drape_precondition","AX_cap_observation_overrun_s":receipt["AX_observation_overrun_s"],"full181_state_and_0_60_180_gates":"failed_or_not_run_incomplete","overall_quality":"not_qualified_incomplete_prefix_only","internal_worker_frames":"unknown; never infer from displayed accepted snapshot"}
    save(OUTPUT/"native-available.json",result)


if __name__=="__main__":
    assert len(sys.argv)==4,"usage: qualify_prefix.py ARTIFACT_ROOT NEW_OUTPUT_DIRECTORY NATIVE_BASE_DIRECTORY"
    OUTPUT.mkdir()
    codes=[analyze_case(name) for name in ("base","height")]
    native_available()
    save(OUTPUT/"scope.json",{"kind":"Frozen post-failure reader results, not prospective physical acceptance","status":"incomplete_prefix_only","arithmetic_consistency_on_available_prefixes":all(code==0 for code in codes),"missing_or_failed_full_gates":["headless_A_180","headless_B_180","both_full181_state_gates","completed-case_records","native_A_180","native_B_not_run","final_cross_case_curve_t"],"overall_quality":"not_qualified_incomplete_prefix_only","new_scenes_steps_retries_extensions":False})
    raise SystemExit(0 if not any(codes) else 1)
