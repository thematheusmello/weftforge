"""Independent geometric material-axis measurement; no solver imports."""
import math


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


if __name__ == "__main__":
    self_check()
    print("Independent axes, energy, weighted-drift, seam and capsule self-checks passed.")
