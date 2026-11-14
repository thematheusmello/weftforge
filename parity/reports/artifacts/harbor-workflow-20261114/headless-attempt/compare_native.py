"""Read-only identical-input native/headless checkpoint comparison."""
from pathlib import Path
import hashlib
import json
import sys
import metrics

ROOT=Path(__file__).resolve().parent
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
read=lambda p:json.loads(p.read_text())


def main():
    name,directory=sys.argv[1],Path(sys.argv[2]).resolve()
    assert name in ("base","height")
    receipt=read(directory/"case-receipt.json")
    assert receipt["case"]==name
    assert receipt["input_sha256"]==sha(ROOT/f"inputs/{name}.weftforge")
    assert receipt["native_runtime_sha256"]=="95fa1b7cb24b8de1a6b507330c2fe43ece0efe5a14226bb28c39f922fcee28c3"
    assert receipt["fresh_original_arrangement"] is True and receipt["initial_zero_velocity"] is True
    assert receipt["dt_s"]==1/60 and receipt["substeps"]==8 and receipt["iterations"]==32 and receipt["seam_closure_m_per_s"]==1.
    assert receipt["assembly_frames"]==[1,60] and receipt["gravity_frames"]==[61,180] and receipt["gravity_m_per_s2"]==[0.,-9.81,0.]
    assert receipt["all_contacts_enabled"] is True and receipt["no_continuation_or_retry"] is True
    meta=read(ROOT/f"runs/{name}/rest-metadata.json")
    states=[json.loads(line) for line in (ROOT/f"runs/{name}/trajectory.jsonl").read_text().splitlines()]
    assert len(states)==181 and read(ROOT/f"runs/{name}/analysis.json")["status"]=="verified"
    excluded=metrics.exclusion(meta)
    rows=[]
    for frame in (0,60,180):
        path=directory/f"stage-{frame:03}.obj"
        checked=metrics.check_obj(meta,states[frame]["positions_m"],path)
        native_positions=[[float(token) for token in row] for row in metrics.parse_obj(path)["v"]]
        strain=metrics.measure(meta,native_positions)
        residuals=metrics.residuals(meta,native_positions)
        contact=metrics.supported_self_contact(meta,native_positions,excluded)
        contact_max=max(residuals["avatar_ground_penetration_m"],contact["supported_static_self_interpanel_penetration_m"])
        rows.append({"frame":frame,"native_obj_sha256":sha(path),"comparison":checked,"independent_native_obj_strain":strain,"independent_native_obj_seam_avatar_ground":residuals,"independent_native_obj_supported_static_contact":contact,"quality_without_unobservable_velocity_or_assembly":{"material_axis_strain_below_0_15":strain["peak"]["absolute_strain"]<.15,"seam_gap_below_0_005_m":residuals["sampled_stitch_gap_m"]<.005,"supported_contact_below_0_002_m":contact_max<.002}})
    result={"case":name,"identical_input_runtime_settings_receipt_sha256":sha(directory/"case-receipt.json"),"status":"verified","checkpoints":rows,"scope":"Exact per-case decimal position/UV tokens, face topology and groups; normal components within 1e-12. Native UI receipts remain required for settings and construction provenance. No native velocity/force, equilibrium, reference calibration or parity claim."}
    with (ROOT/f"native-{name}-comparison.json").open("x") as out:
        json.dump(result,out,indent=2,allow_nan=False);out.write("\n")
    print(json.dumps({"case":name,"status":"verified","checkpoints":[0,60,180]}))


if __name__=="__main__":
    metrics.self_check()
    main()
