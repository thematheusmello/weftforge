"""Independent geometric material-axis measurement; no solver imports."""
import math
from decimal import Decimal


def add(a, b):
    return [x + y for x, y in zip(a, b)]


def mul(a, scalar):
    return [x * scalar for x in a]


def norm(a):
    return math.sqrt(dot(a, a))


def cross(a, b):
    return [a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0]]


def triangle_point(p, a, b, c):
    # Independently minimize the convex quadratic over the face and three segments.
    ab, ac, ap = sub(b, a), sub(c, a), sub(p, a)
    aa, bb, cc = dot(ab, ab), dot(ab, ac), dot(ac, ac)
    determinant = aa*cc-bb*bb
    candidates = []
    if determinant > 1e-28:
        u = (dot(ap, ab)*cc-dot(ap, ac)*bb)/determinant
        v = (dot(ap, ac)*aa-dot(ap, ab)*bb)/determinant
        if u >= 0 and v >= 0 and u+v <= 1:
            candidates.append(add(a, add(mul(ab, u), mul(ac, v))))
    for first, last in ((a,b),(b,c),(c,a)):
        direction = sub(last, first)
        denominator = dot(direction, direction)
        t = max(0., min(1., dot(sub(p, first), direction)/denominator)) if denominator else 0.
        candidates.append(add(first, mul(direction, t)))
    return min(candidates, key=lambda q: dot(sub(p,q),sub(p,q)))


def masses(meta):
    result = [0.]*len(meta["uv_m"])
    pieces = {p["id"]:p for p in meta["project"]["pieces"]}
    materials = {m["id"]:m for m in meta["project"]["materials"]}
    for tri in meta["triangles"]:
        a,b,c = [meta["uv_m"][i] for i in tri]
        x,y = sub(b,a),sub(c,a)
        area = abs(x[0]*y[1]-x[1]*y[0])/2
        contribution = area*materials[pieces[meta["piece_ids"][tri[0]]]["material"]]["areal_density"]/3
        for i in tri:
            result[i] += contribution
    return result


def exclusion(meta):
    excluded = [set() for _ in meta["uv_m"]]
    for tri in meta["triangles"]:
        for i in tri:
            excluded[i].update(tri)
    for stitch in meta["stitches"]:
        for a in stitch["a"]["vertices"]:
            for b in stitch["b"]["vertices"]:
                excluded[a].add(b)
                excluded[b].add(a)
    return excluded


def supported_self_contact(meta, positions, excluded, *, exhaustive=False):
    pieces = {p["id"]:p for p in meta["project"]["pieces"]}
    materials = {m["id"]:m for m in meta["project"]["materials"]}
    thickness = [materials[pieces[pid]["material"]]["collision_thickness"] for pid in meta["piece_ids"]]
    rows = []
    degenerate = 0
    gap = 2*max(thickness)
    for ti, tri in enumerate(meta["triangles"]):
        a,b,c = [positions[i] for i in tri]
        if norm(cross(sub(b,a),sub(c,a))) <= 1e-14:
            degenerate += 1
            continue
        low = [min(a[j],b[j],c[j])-gap for j in range(3)]
        high = [max(a[j],b[j],c[j])+gap for j in range(3)]
        rows.append((ti,tri,a,b,c,low,high,max(thickness[i] for i in tri)))
    cell=max(meta["project"].get("resolution",.07),gap)
    bins={}
    wide=[]
    if not exhaustive:
        for index,row in enumerate(rows):
            low,high=row[5],row[6]
            first=[math.floor(q/cell) for q in low]
            last=[math.floor(q/cell) for q in high]
            count=math.prod(b-a+1 for a,b in zip(first,last))
            # Large triangle boxes fall back to direct checks, so partition size never drops a pair.
            if count>4096:
                wide.append(index)
                continue
            for i in range(first[0],last[0]+1):
                for j in range(first[1],last[1]+1):
                    for k in range(first[2],last[2]+1):
                        bins.setdefault((i,j,k),[]).append(index)
    maximum = 0.
    worst = None
    counts = {"self":0,"interpanel":0}
    intersections = 0
    for vi,p in enumerate(positions):
        candidates=range(len(rows)) if exhaustive else sorted(bins.get(tuple(math.floor(q/cell) for q in p),[])+wide)
        for index in candidates:
            ti,tri,a,b,c,lo,hi,other_thickness=rows[index]
            if not exhaustive and (p[0]<lo[0] or p[0]>hi[0] or p[1]<lo[1] or p[1]>hi[1] or p[2]<lo[2] or p[2]>hi[2]):
                continue
            if any(i in excluded[vi] for i in tri):
                continue
            depth = max(0., thickness[vi]+other_thickness-norm(sub(p,triangle_point(p,a,b,c))))
            if depth > 0:
                kind = "self" if meta["piece_ids"][vi] == meta["piece_ids"][tri[0]] else "interpanel"
                counts[kind] += 1
                intersections += depth > 1e-8
                if depth > maximum:
                    maximum, worst = depth,{"vertex":vi,"triangle":ti,"kind":kind,"depth_m":depth}
    return {"supported_static_self_interpanel_penetration_m":maximum,"positive_depth_pairs":counts,"intersection_pairs_depth_over_1e_minus8":intersections,"worst":worst,"degenerate_faces_skipped_by_supported_model":degenerate,"scope":"Current particle/triangle distances after one-ring/direct-stitch exclusions; no swept or edge-edge completeness claim"}


def analytic_contacts(meta, positions):
    pieces = {p["id"]:p for p in meta["project"]["pieces"]}
    materials = {m["id"]:m for m in meta["project"]["materials"]}
    maximum,intersections=0.,0
    for p,pid in zip(positions,meta["piece_ids"]):
        offset=materials[pieces[pid]["material"]]["collision_thickness"]
        depths=[offset-p[1]]
        for capsule in meta["project"]["mannequin"]["capsules"]:
            a,b=capsule["a"],capsule["b"]
            d=sub(b,a)
            t=max(0.,min(1.,dot(sub(p,a),d)/dot(d,d))) if dot(d,d) else 0.
            depths.append(capsule["radius"]+offset-norm(sub(p,add(a,mul(d,t)))))
        maximum=max(maximum,*depths)
        intersections+=sum(depth>1e-8 for depth in depths)
    return {"avatar_ground_penetration_m":maximum,"intersection_pairs_depth_over_1e_minus8":intersections}


def signed_triangle_axes(meta, positions, index):
    tri=meta["triangles"][index]
    uv=[meta["uv_m"][i] for i in tri]
    a,b=sub(uv[1],uv[0]),sub(uv[2],uv[0])
    determinant=a[0]*b[1]-b[0]*a[1]
    e,f=sub(positions[tri[1]],positions[tri[0]]),sub(positions[tri[2]],positions[tri[0]])
    u=[(e[j]*b[1]-f[j]*a[1])/determinant for j in range(3)]
    v=[(f[j]*a[0]-e[j]*b[0])/determinant for j in range(3)]
    piece=next(p for p in meta["project"]["pieces"] if p["id"]==meta["piece_ids"][tri[0]])
    sine,cosine=math.sin(piece["grain_angle"]),math.cos(piece["grain_angle"])
    return {"warp":norm(add(mul(u,cosine),mul(v,sine)))-1.,"weft":norm(add(mul(u,-sine),mul(v,cosine)))-1.}


def curve_point(meta, positions, piece_id, curve_id, t):
    samples=sorted((s["t"],s["vertex"]) for s in meta["boundary"] if s["piece"]==piece_id and s["curve"]==curve_id)
    assert samples and 0.<=t<=1.
    for fraction,index in samples:
        if fraction==t:
            return positions[index]
    for (left,a),(right,b) in zip(samples,samples[1:]):
        if left<=t<=right:
            weight=(t-left)/(right-left)
            return add(mul(positions[a],1-weight),mul(positions[b],weight))
    raise AssertionError("Authored curve/t is outside the retained boundary map")


def parse_obj(path):
    rows = {"v":[],"vt":[],"vn":[],"f":[],"g":[]}
    group = None
    face_pieces = []
    for line in path.read_text().splitlines():
        tokens = line.split()
        if not tokens or tokens[0].startswith("#"):
            continue
        kind = tokens.pop(0)
        assert kind in rows, (path,kind)
        if kind in ("v","vt","vn"):
            assert len(tokens) == (2 if kind == "vt" else 3)
            assert all(Decimal(token).is_finite() for token in tokens)
            rows[kind].append(tokens)
        elif kind == "g":
            assert len(tokens)==1 and tokens[0].startswith("piece_")
            group = int(tokens[0][6:])
            rows[kind].append(tokens[0])
        else:
            assert len(tokens)==3 and group is not None
            face = []
            for token in tokens:
                indices = list(map(int,token.split("/")))
                assert len(indices)==3 and indices[0]==indices[1]==indices[2] and indices[0]>0
                face.append(indices[0]-1)
            rows[kind].append(face)
            face_pieces.append(group)
    assert len(rows["v"])==len(rows["vt"])==len(rows["vn"])
    assert all(max(tri)<len(rows["v"]) for tri in rows["f"])
    rows["face_pieces"] = face_pieces
    return rows


def expected_obj(meta, positions):
    return {"v":[[format(q,".17f") for q in p] for p in positions],"vt":[[format(q,".17f") for q in p] for p in meta["uv_m"]],"f":meta["triangles"],"face_pieces":[meta["piece_ids"][t[0]] for t in meta["triangles"]]}


def check_obj(meta, positions, path):
    actual = parse_obj(path)
    expected = expected_obj(meta,positions)
    for key,value in expected.items():
        assert actual[key]==value, (path,key,"Expected exact per-case tokens/topology")
    maximum = Decimal(0)
    for tokens,p in zip(actual["v"],positions):
        for token,value in zip(tokens,p):
            error = abs(Decimal(token)-Decimal.from_float(value))
            assert error <= Decimal("5e-18"), (path,token,value,error)
            maximum = max(maximum,error)
    normal_acc = [[0.]*3 for _ in positions]
    for tri in meta["triangles"]:
        a,b,c = [positions[i] for i in tri]
        n = cross(sub(b,a),sub(c,a))
        for i in tri:
            normal_acc[i] = add(normal_acc[i],n)
    maximum_normal_error = 0.
    for tokens,n in zip(actual["vn"],normal_acc):
        length = norm(n)
        assert length>1e-14, "Collapsed display normal"
        maximum_normal_error = max(maximum_normal_error,max(abs(float(t)-q/length) for t,q in zip(tokens,n)))
    assert maximum_normal_error <= 1e-12
    return {"exact_position_uv_face_group_tokens":True,"max_decimal_rounding_error_m":str(maximum),"rounding_bound_m":"5e-18","max_normal_component_error":maximum_normal_error}


def sub(a, b):
    return [x - y for x, y in zip(a, b)]


def dot(a, b):
    return sum(x * y for x, y in zip(a, b))


def drift(a, b, mass):
    assert len(a) == len(b) == len(mass)
    squared = [dot(sub(x, y), sub(x, y)) for x, y in zip(a, b)]
    return {"max_m": math.sqrt(max(squared)), "mass_weighted_rms_m": math.sqrt(sum(m * d for m, d in zip(mass, squared)) / sum(mass))}


def measure(meta, positions):
    pieces = {p["id"]: p for p in meta["project"]["pieces"]}
    materials = {m["id"]: m for m in meta["project"]["materials"]}
    peak = None
    extension, compression, energies = -math.inf, math.inf, [0., 0., 0.]
    for index, triangle in enumerate(meta["triangles"]):
        piece = pieces[meta["piece_ids"][triangle[0]]]
        material = materials[piece["material"]]
        uv = [meta["uv_m"][v] for v in triangle]
        world = [positions[v] for v in triangle]
        a, b = sub(uv[1], uv[0]), sub(uv[2], uv[0])
        det = a[0] * b[1] - b[0] * a[1]
        assert abs(det) > 1e-12
        e, f = sub(world[1], world[0]), sub(world[2], world[0])
        u = [(e[j] * b[1] - f[j] * a[1]) / det for j in range(3)]
        v = [(f[j] * a[0] - e[j] * b[0]) / det for j in range(3)]
        sine, cosine = math.sin(piece["grain_angle"]), math.cos(piece["grain_angle"])
        warp = [cosine * x + sine * y for x, y in zip(u, v)]
        weft = [-sine * x + cosine * y for x, y in zip(u, v)]
        signed = [math.sqrt(dot(axis, axis)) - 1. for axis in [warp, weft]]
        for axis, value in enumerate(signed):
            extension, compression = max(extension, value), min(compression, value)
            if peak is None or abs(value) > peak["absolute_strain"]:
                peak = {"triangle_index": index, "piece_id": piece["id"], "axis": ["warp", "weft"][axis], "stiffness_N_per_m": [material["warp_stiffness"], material["weft_stiffness"]][axis], "vertices_0based": triangle, "signed_strain": value, "absolute_strain": abs(value)}
        constraints = [(dot(warp, warp) - 1) / 2, (dot(weft, weft) - 1) / 2, dot(warp, weft)]
        for j, stiffness in enumerate([material["warp_stiffness"], material["weft_stiffness"], material["shear_stiffness"]]):
            energies[j] += abs(det) * stiffness * constraints[j] ** 2 / 4
    return {"peak": peak, "maximum_signed_extension": extension, "minimum_signed_compression": compression, "membrane_energy_J": dict(zip(["warp", "weft", "shear", "total"], energies + [sum(energies)]))}


def residuals(meta, positions):
    pieces = {p["id"]: p for p in meta["project"]["pieces"]}
    materials = {m["id"]: m for m in meta["project"]["materials"]}
    def point(endpoint):
        a, b = [positions[v] for v in endpoint["vertices"]]
        return [(1. - endpoint["weight"]) * x + endpoint["weight"] * y for x, y in zip(a, b)]
    seam = max((math.sqrt(dot(sub(point(s["a"]), point(s["b"])), sub(point(s["a"]), point(s["b"])))) for s in meta["stitches"]), default=0.)
    contact = 0.
    for p, pid in zip(positions, meta["piece_ids"]):
        thickness = materials[pieces[pid]["material"]]["collision_thickness"]
        contact = max(contact, thickness - p[1])
        for capsule in meta["project"]["mannequin"]["capsules"]:
            a, b = capsule["a"], capsule["b"]
            ab = sub(b, a)
            denominator = dot(ab, ab)
            t = max(0., min(1., dot(sub(p, a), ab) / denominator)) if denominator else 0.
            closest = [x + t * y for x, y in zip(a, ab)]
            delta = sub(p, closest)
            contact = max(contact, thickness + capsule["radius"] - math.sqrt(dot(delta, delta)))
    return {"sampled_stitch_gap_m": seam, "avatar_ground_penetration_m": contact}


def self_check():
    meta = {"project": {"pieces": [{"id": 1, "material": 2, "grain_angle": math.pi / 4}], "materials": [{"id": 2, "warp_stiffness": 50., "weft_stiffness": 1000., "shear_stiffness": 100.}]}, "piece_ids": [1, 1, 1], "triangles": [[0, 1, 2]], "uv_m": [[0., 0.], [1., 0.], [0., 1.]]}
    result = measure(meta, [[0., 0., 0.], [1.2, 0., 0.], [0., 1., 0.]])
    expected = math.sqrt((1.2 ** 2 + 1.) / 2.) - 1.
    assert abs(result["maximum_signed_extension"] - expected) < 1e-15
    assert abs(result["minimum_signed_compression"] - expected) < 1e-15
    meta["project"]["pieces"][0]["grain_angle"] = 0.
    result = measure(meta, [[0., 0., 0.], [1.2, 0., 0.], [0., 1., 0.]])
    assert abs(result["peak"]["signed_strain"] - .2) < 1e-15
    assert result["peak"]["axis"] == "warp"
    assert abs(result["membrane_energy_J"]["warp"] - .5 * 50. * .22 ** 2 / 2.) < 1e-15
    assert drift([[0., 0., 0.], [2., 0., 0.]], [[0., 0., 0.], [0., 0., 0.]], [3., 1.]) == {"max_m": 2., "mass_weighted_rms_m": 1.}
    meta["project"]["materials"][0]["collision_thickness"] = .1
    meta["project"]["mannequin"] = {"capsules": [{"a": [0., 1., 0.], "b": [0., 2., 0.], "radius": .5}]}
    meta["stitches"] = [{"a": {"vertices": [0, 1], "weight": .5}, "b": {"vertices": [1, 2], "weight": .5}}]
    checked = residuals(meta, [[0., 1., 0.], [2., 1., 0.], [0., 2., 0.]])
    assert checked == {"sampled_stitch_gap_m": .5, "avatar_ground_penetration_m": .6}
    assert norm(sub(triangle_point([.25,.25,2.],[0.,0.,0.],[1.,0.,0.],[0.,1.,0.]),[.25,.25,0.]))<1e-15
    assert triangle_point([2.,0.,0.],[0.,0.,0.],[1.,0.,0.],[0.,1.,0.])==[1.,0.,0.]
    assert norm(sub(triangle_point([.5,-1.,0.],[0.,0.,0.],[1.,0.,0.],[0.,1.,0.]),[.5,0.,0.]))<1e-15
    contact_meta = {"project":{"pieces":[{"id":1,"material":1},{"id":2,"material":1}],"materials":[{"id":1,"collision_thickness":.003,"areal_density":.2}]},"uv_m":[[0.,0.],[1.,0.],[0.,1.],[.25,.25],[.5,.25],[.25,.5]],"triangles":[[0,1,2],[3,4,5]],"piece_ids":[1,1,1,2,2,2],"stitches":[]}
    contact_positions = [[0.,0.,0.],[1.,0.,0.],[0.,1.,0.],[.25,.25,.004],[.5,.25,.004],[.25,.5,.004]]
    contact = supported_self_contact(contact_meta,contact_positions,exclusion(contact_meta))
    assert abs(contact["supported_static_self_interpanel_penetration_m"]-.002)<1e-15 and contact["positive_depth_pairs"]["interpanel"]==3
    assert contact==supported_self_contact(contact_meta,contact_positions,exclusion(contact_meta),exhaustive=True)
    for offset in ([0.,0.,0.],[-.07,.14,-.21],[.07,-.07,.07]):
        shifted=[add(p,offset) for p in contact_positions]
        assert supported_self_contact(contact_meta,shifted,exclusion(contact_meta))==supported_self_contact(contact_meta,shifted,exclusion(contact_meta),exhaustive=True)
    wide_meta={**contact_meta,"project":{**contact_meta["project"],"resolution":.0001}}
    wide_positions=[mul(p,100.) for p in contact_positions]
    assert supported_self_contact(wide_meta,wide_positions,exclusion(wide_meta))==supported_self_contact(wide_meta,wide_positions,exclusion(wide_meta),exhaustive=True)
    stitched_meta={**contact_meta,"stitches":[{"a":{"vertices":[0,1],"weight":.5},"b":{"vertices":[3,4],"weight":.5}}]}
    assert supported_self_contact(stitched_meta,contact_positions,exclusion(stitched_meta))==supported_self_contact(stitched_meta,contact_positions,exclusion(stitched_meta),exhaustive=True)
    self_meta={**contact_meta,"piece_ids":[1]*6}
    assert supported_self_contact(self_meta,contact_positions,exclusion(self_meta))==supported_self_contact(self_meta,contact_positions,exclusion(self_meta),exhaustive=True)
    assert abs(sum(masses(contact_meta))-.10625)<1e-15
    assert analytic_contacts(meta,[[0.,1.,0.],[2.,1.,0.],[0.,2.,0.]])["avatar_ground_penetration_m"]==.6
    obj_meta={"uv_m":[[0.,0.],[1.,0.],[0.,1.]],"triangles":[[0,1,2]],"piece_ids":[1,1,1]}
    obj_positions=[[0.,0.,0.],[1.,0.,0.],[0.,1.,0.]]
    class ObjFixture:
        def read_text(self):
            expected=expected_obj(obj_meta,obj_positions)
            rows=["v "+" ".join(row) for row in expected["v"]]+["vt "+" ".join(row) for row in expected["vt"]]
            return "\n".join(rows+["vn 0 0 1"]*3+["g piece_1","f 1/1/1 2/2/2 3/3/3"])
    assert check_obj(obj_meta,obj_positions,ObjFixture())["exact_position_uv_face_group_tokens"]
    class BadObjFixture(ObjFixture):
        def read_text(self):
            return super().read_text().replace("f 1/1/1", "f 0/0/0")
    try:
        parse_obj(BadObjFixture())
    except AssertionError:
        pass
    else:
        raise AssertionError("OBJ parser accepted invalid external face indices")


if __name__ == "__main__":
    self_check()
    print("Independent axes, energy, weighted-drift, seam and capsule self-checks passed.")
