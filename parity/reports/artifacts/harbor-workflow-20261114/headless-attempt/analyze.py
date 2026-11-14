"""Independent source-formula checks; quality failure remains a result, never a retry trigger."""
from pathlib import Path
import datetime
import hashlib
import json
import math
import sys
import metrics

ROOT=Path(__file__).resolve().parent
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()


def load(path):
    return json.loads(path.read_text(),parse_constant=lambda value: (_ for _ in ()).throw(ValueError(f"Non-finite JSON token: {value}")))


def lines(path):
    return [json.loads(line) for line in path.read_text().splitlines()]


def save(path, value):
    with path.open("x") as out:
        json.dump(value,out,indent=2,allow_nan=False);out.write("\n")


def exact_rest(actual,expected):
    a=dict(actual)
    a.pop("masses_kg")
    assert a==expected,"Actual solver rest metadata differs from the frozen geometry-only snapshot"


def assert_close(a,b,tolerance,label):
    assert math.isfinite(a) and math.isfinite(b) and abs(a-b)<=tolerance,(label,a,b,tolerance)


def quality(strain,seam,contact,remaining,finite=True):
    return {"finite":finite,"completed_assembly":remaining==0.,"material_axis_strain_below_0_15":strain<.15,"sampled_seam_gap_below_0_005_m":seam<.005,"supported_contact_penetration_below_0_002_m":contact<.002}


def analyze_case(name):
    assert name in ("base","height")
    run=ROOT/"runs"/name
    result={"kind":"Independent original UI-authored Harbor implementation-only physical verification","case":name,"status":"failed","reference_parity":"not measured","equilibrium_claim":False,"quality_passed":False,"rows":[]}
    try:
        registration=load(ROOT/"preregistration.json")
        for path,expected in registration["frozen_files"].items():
            assert sha(ROOT/path)==expected,path
        meta=load(run/"rest-metadata.json")
        exact_rest(meta,load(ROOT/f"prepared/{name}/mesh-metadata.json"))
        expected_masses=metrics.masses(meta)
        assert len(expected_masses)==len(meta["masses_kg"])
        mass_error=max(abs(a-b) for a,b in zip(expected_masses,meta["masses_kg"]))
        assert mass_error<=1e-12
        result["mass_check"]={"max_nodal_error_kg":mass_error,"independent_total_kg":sum(expected_masses),"solver_total_kg":sum(meta["masses_kg"])}
        states,steps=lines(run/"trajectory.jsonl"),lines(run/"steps.jsonl")
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
                assert steps[frame-1]["diagnostics"]==state["diagnostics"]
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
            row={"frame":frame,"diagnostics":d,"independent_material_axis":strain,"independent_seam":residuals["sampled_stitch_gap_m"],"independent_analytic_contact":analytic,"independent_supported_static_contact":contact,"quality_gates":gates,"quality_passed":all(gates.values()),"rms_speed_m_per_s":math.sqrt(2*kinetic/sum(expected_masses)),"max_speed_m_per_s":max(map(metrics.norm,v))}
            if frame in (60,180):
                row["obj_check"]=metrics.check_obj(meta,x,run/f"stage-{frame:03}.obj")
            result["rows"].append(row)
        complete=load(run/"completed-case.json")
        assert len(states)==181 and len(steps)==180 and complete["completed_frames"]==180
        assert complete["final_diagnostics"]==states[-1]["diagnostics"]
        source_gates=quality(d["max_stretch_error"],d["max_seam_error"],d["max_contact_error"],d["seam_assembly_remaining"],d["finite"])
        assert complete["original_quality_passed"]==all(source_gates.values())
        result.update(status="verified",completed_frames=180,quality_passed=result["rows"][-1]["quality_passed"] and complete["original_quality_passed"],solver_original_quality_gates=source_gates,all_original_contact_enabled=True)
    except Exception as exc:
        result["first_failure"]=f"{type(exc).__name__}: {exc}"
    result["completed_utc"]=datetime.datetime.now(datetime.timezone.utc).isoformat()
    save(run/"analysis.json",result)
    print(json.dumps({key:result.get(key) for key in ("case","status","quality_passed","completed_frames","first_failure")}),flush=True)
    return 0 if result["status"]=="verified" else 1


def edit_response():
    metas={name:load(ROOT/f"prepared/{name}/mesh-metadata.json") for name in ("base","height")}
    states={name:lines(ROOT/f"runs/{name}/trajectory.jsonl") for name in metas}
    assert all(len(v)==181 for v in states.values())
    rows=[]
    for frame in (0,60,180):
        pieces=[]
        for pid,first_curve in ((2,17),(31,46)):
            samples=[]
            for ordinal in range(14):
                for t in (0.,.25,.5,.75,1.):
                    positions={name:metrics.curve_point(metas[name],states[name][frame]["positions_m"],pid,first_curve+ordinal,t) for name in metas}
                    distance=metrics.norm(metrics.sub(positions["height"],positions["base"]))
                    samples.append({"boundary_index":ordinal,"t":t,"base_m":positions["base"],"height_m":positions["height"],"distance_m":distance})
            pieces.append({"piece_id":pid,"max_corresponding_curve_t_displacement_m":max(s["distance_m"] for s in samples),"samples":samples})
        rows.append({"frame":frame,"pieces":pieces})
    assert abs(rows[0]["pieces"][0]["max_corresponding_curve_t_displacement_m"]-.04)<=1e-12
    assert rows[0]["pieces"][1]["max_corresponding_curve_t_displacement_m"]<=1e-12
    result={"kind":"Authored-curve/t comparison across different meshes; no nodal comparison across cases","stages":rows,"positive_front_edit_in_rest_geometry":rows[0]["pieces"][0]["max_corresponding_curve_t_displacement_m"]>1e-12,"positive_front_edit_response_in_simulated_3d_at_60_and_180":all(r["pieces"][0]["max_corresponding_curve_t_displacement_m"]>1e-12 for r in rows[1:]),"limitation":"Observed coupled response to authored edit and resulting remesh; this is not mesh convergence, derivative accuracy, stability or physical/reference parity."}
    save(ROOT/"edit-response.json",result)
    print(json.dumps({k:v for k,v in result.items() if k!="stages"}))
    return 0


if __name__=="__main__":
    metrics.self_check()
    if sys.argv[1]=="self-check":
        assert all(quality(.149,.004,.001,0.).values())
        for values in ((.15,.004,.001,0.),(.149,.005,.001,0.),(.149,.004,.002,0.),(.149,.004,.001,1e-16)):
            assert not all(quality(*values).values())
        assert not all(quality(.149,.004,.001,0.,False).values())
        print("Independent metric/parser and strict unchanged quality boundary self-checks passed.")
        raise SystemExit(0)
    raise SystemExit(edit_response() if sys.argv[1]=="compare-edit" else analyze_case(sys.argv[1]))
