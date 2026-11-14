"""Geometry-only timing: all181 evaluations on each mesh and synthetic close flat layers."""
from pathlib import Path
import datetime
import hashlib
import json
import os
import platform
import time
import metrics

ROOT=Path(__file__).resolve().parent
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
started=time.monotonic()
record={"kind":"Preparation-only static geometry evaluator throughput; no Cloth or stepping","pid":os.getpid(),"started_utc":datetime.datetime.now(datetime.timezone.utc).isoformat(),"python":platform.python_version(),"machine":platform.machine(),"metrics_sha256":sha(ROOT/"metrics.py"),"benchmark_source_sha256":sha(Path(__file__)),"coverage":"181 complete independent metric sets for each mesh and static geometry arrangement; no trajectory is fabricated","hard_wall_cap_seconds":120,"jobs":[]}
with (ROOT/"checks/evaluator-benchmark-start.json").open("x") as out:
    json.dump(record,out,indent=2);out.write("\n")
print(json.dumps({"pid":os.getpid(),"kind":record["kind"]}),flush=True)
metrics.self_check()
for name in ("base","height"):
    meta=json.loads((ROOT/f"prepared/{name}/mesh-metadata.json").read_text())
    excluded=metrics.exclusion(meta)
    for arrangement in ("rest","synthetic_parallel_layers_4mm_apart"):
        positions=meta["rest_positions_m"] if arrangement=="rest" else [[p[0],p[1],0. if pid==2 else .004] for p,pid in zip(meta["rest_positions_m"],meta["piece_ids"])]
        beginning=time.monotonic()
        worst={}
        for boundary in range(181):
            if time.monotonic()-started>=120:
                record.update(status="stopped_at_cap",elapsed_seconds=time.monotonic()-started)
                with (ROOT/"checks/evaluator-benchmark.json").open("x") as out:
                    json.dump(record,out,indent=2);out.write("\n")
                raise SystemExit(124)
            measured=metrics.measure(meta,positions)
            residuals=metrics.residuals(meta,positions)
            analytic=metrics.analytic_contacts(meta,positions)
            contact=metrics.supported_self_contact(meta,positions,excluded)
            worst={"material_axis_strain":measured["peak"]["absolute_strain"],"sampled_seam_gap_m":residuals["sampled_stitch_gap_m"],"avatar_ground_penetration_m":analytic["avatar_ground_penetration_m"],"supported_self_interpanel_penetration_m":contact["supported_static_self_interpanel_penetration_m"],"positive_depth_pairs":contact["positive_depth_pairs"]}
        row={"mesh_case":name,"arrangement":arrangement,"vertices":len(positions),"triangles":len(meta["triangles"]),"complete_independent_metric_sets":181,"wall_seconds":time.monotonic()-beginning,"last_static_metric_set":worst}
        record["jobs"].append(row)
        print(json.dumps(row),flush=True)
record.update(status="completed",elapsed_seconds=time.monotonic()-started,physical_solver_throughput="not measured or promised; each real driver+analysis remains hard-capped at420s",worst_case_geometry_completion_guarantee=False,acceleration_correctness="Exhaustive-vs-grid analytic selfchecks include face/edge/vertex regions, shifted grid boundaries, self/interpanel, seam exclusions and wide-AABB fallback; partition never drops wide triangles")
with (ROOT/"checks/evaluator-benchmark.json").open("x") as out:
    json.dump(record,out,indent=2);out.write("\n")
print(json.dumps({"status":record["status"],"elapsed_seconds":record["elapsed_seconds"]}),flush=True)
