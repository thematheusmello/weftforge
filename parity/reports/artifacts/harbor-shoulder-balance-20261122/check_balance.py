"""Reconstruct reduced shoulder force balances using only the Python standard library."""
import hashlib
import json
import math
from pathlib import Path

HERE = Path(__file__).resolve().parent


def sub(a, b):
    return tuple(x-y for x, y in zip(a, b))


def dot(a, b):
    return sum(x*y for x, y in zip(a, b))


def cross(a, b):
    return (a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0])


def normalized(v):
    length = math.hypot(*v)
    return tuple(x/length for x in v)


def material_vectors(p, chart):
    b, c = sub(p[1], p[0]), sub(p[2], p[0])
    (u,v), (s,t) = sub(chart[1], chart[0]), sub(chart[2], chart[0])
    det = u*t-v*s
    return ([(-s*x+u*y)/det for x, y in zip(b,c)],
            [(t*x-v*y)/det for x, y in zip(b,c)], abs(det)/2)


def bend(p):
    a,b,c,d = p
    edge = sub(b,a)
    n1, n2 = cross(edge,sub(c,a)), cross(sub(d,a),edge)
    angle = math.atan2(dot(cross(n1,n2),normalized(edge)),dot(n1,n2))
    return angle, 2*dot(edge,edge)/(math.hypot(*n1)+math.hypot(*n2))


def segment_distance(p,q,a,b):
    u,v,w = sub(q,p),sub(b,a),sub(p,a)
    A,B,C,D,E = dot(u,u),dot(u,v),dot(v,v),dot(u,w),dot(v,w)
    clamp = lambda t: max(0., min(1.,t))
    choices = [(0,clamp(E/C)), (1,clamp((B+E)/C)),
               (clamp(-D/A),0), (clamp((B-D)/A),1)]
    det = A*C-B*B
    if det > 1e-14*A*C:
        h,k = (B*E-C*D)/det, (A*E-B*D)/det
        if 0 <= h <= 1 and 0 <= k <= 1:
            choices.append((h,k))
    return min(math.hypot(*[d+h*u0-k*v0 for d,u0,v0 in zip(w,u,v)]) for h,k in choices)


def capsule_distance(triangle, cap):
    a,b = cap[:3],cap[3:6]
    p,q,r = triangle
    n = normalized(cross(sub(q,p),sub(r,p)))
    edges = ((p,q),(q,r),(r,p))
    def inside(point):
        return all(dot(cross(sub(y,x),sub(point,x)),n) >= 0 for x,y in edges)
    da,db = dot(n,sub(a,p)),dot(n,sub(b,p))
    if da*db < 0:
        t = da/(da-db)
        if inside([v+t*(w-v) for v,w in zip(a,b)]):
            return 0.
    distances = [segment_distance(x,y,a,b) for x,y in edges]
    for end,d in ((a,da),(b,db)):
        if inside(end):
            distances.append(abs(d))
    return min(distances)


def grade(data, name):
    mat = data["material"]
    assert abs(mat["grain_angle"] - math.pi/2) < 1e-12
    row = data["cases"][name]
    nodes = row["mobile"]
    x = [list(p) for p in data["current"]]
    for i,p in zip(nodes,row["positions"]):
        x[i] = p.copy()
    panels = [(t, [data["uv"][i] for i in t]) for t in data["triangles"] if set(t)&set(nodes)]
    hinges = [(h,*bend([data["rest"][i] for i in h])) for h in data["hinges"] if set(h)&set(nodes)]

    def energy():
        elastic = []
        for t, uv in panels:
            w,f,area = material_vectors([x[i] for i in t],uv)
            modes = (0.5*(dot(w,w)-1), 0.5*(dot(f,f)-1), dot(w,f))
            moduli = (mat["warp_stiffness"],mat["weft_stiffness"],mat["shear_stiffness"])
            elastic.append(0.5*area*sum(k*s*s for k,s in zip(moduli,modes)))
        for h,rest,scale in hinges:
            angle = bend([x[i] for i in h])[0] - rest
            turn = math.atan2(math.sin(angle),math.cos(angle))
            elastic.append(0.5*mat["bending_stiffness"]*scale*turn**2)
        return math.fsum(elastic)

    # A symmetric five-point difference uses energies independently of native force rows.
    balance = {i:[0.0]*3 for i in nodes}
    for i in nodes:
        for axis in range(3):
            start = x[i][axis]
            eps = 1e-6
            values = []
            for j in (-2,-1,1,2):
                x[i][axis] = start+j*eps
                values.append(energy())
            x[i][axis] = start
            grad = (values[0]-8*values[1]+8*values[2]-values[3])/(12*eps)
            inertia = data["masses"][i]*(start-data["predicted"][i][axis])/data["dt"]**2
            balance[i][axis] = grad + inertia

    tension = []
    native_tension = []
    for sew in data["sews"]:
        ids = sew["coefficients"]
        delta = [sum(w*x[i][a] for i,w in ids) for a in range(3)]
        force = (math.hypot(*delta)-sew["goal_m"])/sew["compliance"]
        tension.append(force)
        native_delta = [sum(w*data["current"][i][a] for i,w in ids) for a in range(3)]
        native_tension.append((math.hypot(*native_delta)-sew["goal_m"])/sew["compliance"])
        for i,w in ids:
            if i in balance:
                for a,u in enumerate(normalized(delta)):
                    balance[i][a] += w*force*u

    max_law = 0.
    for contact in row["contact"]:
        ids, weights, force = contact["vertices"],contact["weights"],contact["force"]
        assert min(weights) >= -1e-12 and abs(sum(weights)-1) < 1e-12
        pos = [sum(w*x[i][a] for i,w in zip(ids,weights)) for a in range(3)]
        old = [sum(w*data["previous"][i][a] for i,w in zip(ids,weights)) for a in range(3)]
        cap = data["capsules"][contact["body"]]
        axis = sub(cap[3:6],cap[:3])
        t = max(0.,min(1.,dot(sub(pos,cap[:3]),axis)/dot(axis,axis)))
        radial = sub(pos,[a+t*b for a,b in zip(cap[:3],axis)])
        normal = normalized(radial)
        gap = math.hypot(*radial)-cap[6]-mat["collision_thickness"]
        pressure = dot(force,normal)
        assert abs(gap) < 1e-9 and pressure > -1e-9
        slip = sub(pos,old)
        tang = [v-dot(slip,normal)*n for v,n in zip(slip,normal)]
        friction = [f-pressure*n for f,n in zip(force,normal)]
        cone = mat["friction"]*pressure
        assert math.hypot(*friction) <= cone+1e-7
        if math.hypot(*tang) > 1e-9:
            max_law = max(max_law, math.hypot(*[f+cone*u for f,u in zip(friction,normalized(tang))]))
        for i,w in zip(ids,weights):
            if i in balance:
                for a in range(3):
                    balance[i][a] -= w*force[a]
    gaps = []
    for tri, _ in panels:
        for cap in data["capsules"]:
            offset = cap[6]+mat["collision_thickness"]
            gap = capsule_distance([x[i] for i in tri], cap)-offset
            baseline = capsule_distance([data["current"][i] for i in tri], cap)-offset
            assert gap >= min(0,baseline)-1e-9
            gaps.append(gap)
    residual = math.hypot(*(v for row_force in balance.values() for v in row_force))
    assert residual < 1e-6 and max_law < 1e-6 and min(gaps) > -1e-9
    first = data["face_ids"].index(6)
    t = data["triangles"][first]
    warp = math.hypot(*material_vectors([x[i] for i in t], [data["uv"][i] for i in t])[0])-1
    diags, contacts, motion = row["native_geometry"]
    assert contacts == row["swept_self_contact_count"] == 0 and motion == 0
    assert diags["finite"] and warp <= diags["max_stretch_error"] + 1e-12
    if diags["peak_strain"]["triangle"] == 6:
        assert abs(warp-diags["max_stretch_error"]) < 1e-12
    assert warp > 0.15 and abs(native_tension[0]) > 5000 and 0 < tension[0] < 3
    return dict(mobile=len(nodes), force_residual_N=residual, warp=warp,
                min_capsule_gap_m=min(gaps), shoulder_tension_N=tension[0],
                native_return_implied_tension_N=native_tension[0])


if __name__=="__main__":
    for case in ("original","extended"):
        data = json.loads((HERE/(case+".json")).read_bytes())
        fixture = HERE.parents[3] / data["fixture"]
        assert hashlib.sha256(fixture.read_bytes()).hexdigest() == data["fixture_sha256"]
        stream = HERE.parent / "harbor-shoulder-20261122" / (case+".jsonl")
        geometry, *records = map(json.loads, stream.read_text().splitlines())
        returned = next(r for r in records if r["frame"] == 28 and r["branch"] == "canonical")
        for ids, triangle in zip(geometry["vertices"], returned["positions"]):
            for i,pos in zip(ids,triangle):
                assert data["current"][data["global_ids"].index(i)] == pos
        for sample in data["balanced_from_rest_samples"]:
            lengths = []
            anchors = []
            for face, chart in zip(sample["positions"], geometry["uv"]):
                weight = (chart[0][0]-chart[1][0])/(chart[2][0]-chart[1][0])
                anchor = [a+weight*(b-a) for a,b in zip(face[1],face[2])]
                anchors.append(anchor)
                lengths.append(math.dist(face[0],anchor))
            gap = math.dist(*[t[0] for t in sample["positions"]])
            bound = (math.dist(*anchors)-gap)/2
            assert max(lengths) >= bound and sample["vertices"] == geometry["vertices"]
            if sample["frame"] >= 28:
                assert max(lengths) > 0.0575 > bound
            print(case, "balanced-from-rest", sample["frame"], sample["substep"], lengths, bound)
        for scope in ("endpoints","faces","ring"):
            print(case, scope, grade(data,scope))
