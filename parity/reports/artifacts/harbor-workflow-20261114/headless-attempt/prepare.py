"""Freeze authoring-derived expectations, then verify geometry without constructing cloth."""
from pathlib import Path
from decimal import Decimal, localcontext
import datetime
import hashlib
import json
import math
import sys
import struct
import metrics

ROOT = Path(__file__).resolve().parent
TOLERANCE = 1e-12
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
read = lambda p: json.loads(p.read_text())


def save(name, value):
    with (ROOT/name).open("x") as out:
        json.dump(value,out,indent=2,allow_nan=False)
        out.write("\n")


def polygon_area(points):
    return abs(sum(a[0]*b[1]-b[0]*a[1] for a,b in zip(points,points[1:]+points[:1])))/2


def analytic():
    plan = read(ROOT/"inputs/authoring-plan.json")
    assert sha(ROOT/"inputs/authoring-plan.json")=="0f07ac196c8e285c1441dc3a16f0027f65c2b901b4630abfa180a216b56e1ddd"
    cases = {}
    with localcontext() as ctx:
        ctx.prec=50
        for name,expected_sha in (("base","fc494974955d440aacbfde49c332a50218bd3a9354d06f4fbbf0596e5da8eda7"),("height","725fc896bc7f020682e2bef3a471a3e1a9048dcab50a7733afad3597722a7ec5")):
            path = ROOT/f"inputs/{name}.weftforge"
            assert sha(path)==expected_sha
            project = read(path)["project"]
            assert len(project["pieces"])==2 and len(project["points"])==len(project["curves"])==28
            assert len(project["seams"])==8 and project["sewing_groups"]==[] and project["resolution"]==.07
            assert project["materials"]==[plan["material"]]
            assert project["next_id"]==68 and project["revision"]=={"base":20,"height":17}[name]
            points={p["id"]:p["position"] for p in project["points"]}
            curves={c["id"]:c for c in project["curves"]}
            pieces=[]
            for ordinal,(piece,planned) in enumerate(zip(project["pieces"],plan["pieces"])):
                assert piece["id"]==[2,31][ordinal] and piece["name"]==planned["name"]
                assert piece["placement"]==planned["placement"] and piece["grain_angle"]==planned["grain_angle"] and piece["material"]==1
                expected=[[Decimal(str(q)) for q in p] for p in planned["vertices"]]
                if name=="height" and ordinal==0:
                    expected=[[p[0],Decimal(".96")+(p[1]-Decimal(".96"))*Decimal(17)/16] for p in expected]
                actual=[]
                for index,cid in enumerate(piece["boundary"]):
                    c=curves[cid]
                    assert cid==[17,46][ordinal]+index and c["controls"] is None
                    assert c["start"]==[3,32][ordinal]+index and c["end"]==[3,32][ordinal]+(index+1)%14
                    actual.append(points[c["start"]])
                error=max(abs(float(q)-a) for p,a2 in zip(expected,actual) for q,a in zip(p,a2))
                assert error<=TOLERANCE
                area=polygon_area(expected)
                pieces.append({"piece_id":piece["id"],"name":piece["name"],"point_ids":[curves[c]["start"] for c in piece["boundary"]],"curve_ids":piece["boundary"],"expected_uv_decimal":[list(map(str,p)) for p in expected],"area_m2_decimal":str(area),"mass_kg_decimal":str(area*Decimal(".18")),"placement":piece["placement"],"material_id":1,"grain_angle_rad":piece["grain_angle"],"max_authored_coordinate_formula_error_m":error,"bounds_uv_m":[[float(min(p[j] for p in expected)) for j in (0,1)],[float(max(p[j] for p in expected)) for j in (0,1)]]})
            seams=[]
            for index,(s,boundary_index) in enumerate(zip(project["seams"],plan["seams_zero_based_matching_boundary_indices"])):
                assert s["id"]==60+index and s["name"]==f"Seam {index+1}"
                assert s["a"]=={"curve":17+boundary_index,"t0":0.,"t1":1.} and s["b"]=={"curve":46+boundary_index,"t0":0.,"t1":1.} and s["correspondence"]==[[0.,0.],[1.,1.]]
                lengths=[]
                for piece in pieces:
                    vertices=[[Decimal(q) for q in p] for p in piece["expected_uv_decimal"]]
                    a,b=vertices[boundary_index],vertices[(boundary_index+1)%14]
                    lengths.append(sum((q-r)**2 for q,r in zip(a,b)).sqrt())
                seams.append({"seam_id":s["id"],"boundary_index_0based":boundary_index,"curve_ids":[s["a"]["curve"],s["b"]["curve"]],"intervals":[[0,1],[0,1]],"fraction_map":"t_b=t_a; same direction","analytic_lengths_m_decimal":list(map(str,lengths)),"length_mismatch_m_decimal":str(abs(lengths[0]-lengths[1])),"compliance_m_per_N":1e-8})
            area=sum(Decimal(p["area_m2_decimal"]) for p in pieces)
            max_dy=Decimal(".04") if name=="height" else Decimal(0)
            max_separation=(Decimal(".48")**2+max_dy**2).sqrt()
            cases[name]={"input":f"inputs/{name}.weftforge","input_sha256":expected_sha,"revision":project["revision"],"pieces":pieces,"seams":seams,"area_m2_decimal":str(area),"mass_kg_decimal":str(area*Decimal(".18")),"maximum_initial_stitch_separation_upper_bound_m_decimal":str(max_separation),"assembly_targets_expected_zero_by_frame":29,"ground_height_m":0.,"avatar_capsules":project["mannequin"]["capsules"],"pins":[],"material":plan["material"]}
    a,b=read(ROOT/"inputs/base.weftforge")["project"],read(ROOT/"inputs/height.weftforge")["project"]
    for key in a:
        if key not in ("points","revision"):
            assert a[key]==b[key],key
    assert a["points"][14:]==b["points"][14:]
    save("analytic-expectations.json",{"kind":"Prospective independent authoring-derived geometry; no cloth constructed or stepped","frozen_utc":datetime.datetime.now(datetime.timezone.utc).isoformat(),"coordinate_tolerance_m":TOLERANCE,"area_formula":"absolute shoelace/2 on exact plan decimals; height front y=.96+(y-.96)*17/16","mass_formula":"areal_density 0.18 kg/m2 times authored area; each material triangle contributes rho*A/3 per incident node","cases":cases,"edit_scope":"Front height only; X within 1e-12 m, Y formula, all back and other construction exact; monotonic revision differs","prior_failed_fixture_unchanged_sha256":"82e7c6d398c7772d7e62c28196dcf11845020020d44fca9ac9113e615b9537e7","quality_prediction":"unknown; ease/clearance rationale is not a quality result"})


def mesh_contracts():
    analytic=read(ROOT/"analytic-expectations.json")
    contracts={}
    for name,case in analytic["cases"].items():
        directory=ROOT/f"prepared/{name}"
        meta=read(directory/"mesh-metadata.json")
        p=meta["project"]
        typed_source=read(ROOT/case["input"])["project"]
        for material in typed_source["materials"]:
            material["color"]=[struct.unpack("f",struct.pack("f",q))[0] for q in material["color"]]
        assert p==typed_source and meta["doc_revision"]==case["revision"]
        points={v["id"]:v["position"] for v in p["points"]}
        curves={v["id"]:v for v in p["curves"]}
        pieces={v["id"]:v for v in p["pieces"]}
        assert len(meta["uv_m"])==len(meta["rest_positions_m"])==len(meta["piece_ids"])
        for uv,x,pid in zip(meta["uv_m"],meta["rest_positions_m"],meta["piece_ids"]):
            t=pieces[pid]["placement"]["translation"]
            assert max(abs(a-b) for a,b in zip(x,[uv[0]+t[0],uv[1]+t[1],t[2]]))<=TOLERANCE
        for sample in meta["boundary"]:
            curve=curves[sample["curve"]]
            a,b=points[curve["start"]],points[curve["end"]]
            expected=[q+(r-q)*sample["t"] for q,r in zip(a,b)]
            assert meta["piece_ids"][sample["vertex"]]==sample["piece"]
            assert max(abs(q-r) for q,r in zip(expected,meta["uv_m"][sample["vertex"]]))<=TOLERANCE
        per_piece={}
        for pid,piece in pieces.items():
            area=0.
            vertices={i for i,v in enumerate(meta["piece_ids"]) if v==pid}
            triangles=[tri for tri in meta["triangles"] if meta["piece_ids"][tri[0]]==pid]
            edges={}
            for tri in triangles:
                assert len(set(tri))==3 and set(tri)<=vertices
                a,b,c=[meta["uv_m"][v] for v in tri]
                e,f=metrics.sub(b,a),metrics.sub(c,a)
                determinant=e[0]*f[1]-e[1]*f[0]
                assert determinant>1e-12
                area+=determinant/2
                for i in range(3):
                    edge=tuple(sorted((tri[i],tri[(i+1)%3])))
                    edges[edge]=edges.get(edge,0)+1
            assert max(edges.values())==2 and len(vertices)-len(edges)+len(triangles)==1
            expected=next(v for v in case["pieces"] if v["piece_id"]==pid)
            assert abs(area-float(expected["area_m2_decimal"]))<=1e-12
            per_piece[str(pid)]={"vertices":len(vertices),"triangles":len(triangles),"edges":len(edges),"boundary_edges":sum(v==1 for v in edges.values()),"material_area_m2":area,"analytic_area_m2":expected["area_m2_decimal"]}
        source_masses=metrics.masses(meta)
        assert abs(sum(source_masses)-float(case["mass_kg_decimal"]))<=1e-12 and min(source_masses)>0
        for t,m in zip(meta["triangles"],meta["triangle_materials"]):
            piece=pieces[meta["piece_ids"][t[0]]]
            expected={key:case["material"][key] for key in ("areal_density","warp_stiffness","weft_stiffness","shear_stiffness","bending_stiffness","damping","friction","collision_thickness")}
            expected.update(piece_id=piece["id"],material_id=1,grain_angle=piece["grain_angle"])
            assert m==expected
        stitch_rows=[]
        for s in meta["stitches"]:
            declared=next(v for v in p["seams"] if v["id"]==s["seam"])
            fractions=[]
            for endpoint,side in ((s["a"],"a"),(s["b"],"b")):
                curve=curves[declared[side]["curve"]]
                a,b=points[curve["start"]],points[curve["end"]]
                x=[(1-endpoint["weight"])*q+endpoint["weight"]*r for q,r in zip(*[meta["uv_m"][i] for i in endpoint["vertices"]])]
                d=metrics.sub(b,a)
                t=metrics.dot(metrics.sub(x,a),d)/metrics.dot(d,d)
                assert 0.-TOLERANCE<=t<=1.+TOLERANCE and metrics.norm(metrics.sub(x,[q+t*r for q,r in zip(a,d)]))<=TOLERANCE
                fractions.append(t)
            assert abs(fractions[0]-fractions[1])<=TOLERANCE
            stitch_rows.append({"seam_id":s["seam"],"authored_fraction":fractions[0]})
        assert set(s["seam_id"] for s in stitch_rows)==set(range(60,68))
        expected=metrics.expected_obj(meta,meta["rest_positions_m"])
        with (directory/"expected-obj-tokens.json").open("x") as out:
            json.dump(expected,out,separators=(",",":"));out.write("\n")
        obj_check=metrics.check_obj(meta,meta["rest_positions_m"],directory/"rest.obj")
        contracts[name]={"metadata_sha256":sha(directory/"mesh-metadata.json"),"obj_token_sha256":sha(directory/"expected-obj-tokens.json"),"rest_obj_sha256":sha(directory/"rest.obj"),"vertices":len(meta["uv_m"]),"triangles":len(meta["triangles"]),"boundary_records":len(meta["boundary"]),"weighted_stitches":len(meta["stitches"]),"per_piece":per_piece,"independent_total_mass_kg":sum(source_masses),"analytic_total_mass_kg":case["mass_kg_decimal"],"minimum_nodal_mass_kg":min(source_masses),"stitch_authored_curve_fraction_map":stitch_rows,"obj_check":obj_check}
    save("mesh-contracts.json",{"kind":"Geometry-only preflight; no Cloth construction or dynamics","cases":contracts,"same_input_native_headless":"Exact topology, piece groups, UV and position decimal tokens expected within each case; native/sim checkpoints must use identical input and settings","cross_edit_comparison":"Topologies may differ. Compare corresponding piece/curve/t and normalized authored polygon/UV material locations, never node indices across the two cases."})


if __name__=="__main__":
    metrics.self_check()
    {"analytic":analytic,"mesh":mesh_contracts}[sys.argv[1]]()
