"""Static triangle/capsule occupancy, independent of all product code."""
import itertools
import math
from decimal import Decimal, localcontext

import decimal_oracle as oracle
import geometry_math as g
import static_geometry as features

EPS = 2.220446049250313e-16


def high_precision_interval(triangle, a, b):
    """Prior oracle active sets; feasible primal/dual witnesses certify its result."""
    with localcontext() as context:
        context.prec = 100
        dec = lambda p: [Decimal.from_float(float(x)) for x in p]
        tri, a, b = list(map(dec, triangle)), dec(a), dec(b)
        u, v, axis, delta = g.sub(tri[1], tri[0]), g.sub(tri[2], tri[0]), g.sub(b, a), g.sub(tri[0], a)
        scale = max(abs(x) for p in (u, v, axis, delta) for x in p)
        if not scale:
            return {"distance_lower_m": 0., "distance_upper_m": 0., "uncertain": False,
                    "method": "decimal100_identical_point", "barycentric": [1., 0., 0.], "axis_fraction": 0.}
        u, v, axis, delta = [[x / scale for x in p] for p in (u, v, axis, delta)]
        columns = [u, v, [-x for x in axis]]
        hessian = [[g.dot(x, y) for y in columns] for x in columns]
        linear = [g.dot(x, delta) for x in columns]
        constraints = [[-1, 0, 0], [0, -1, 0], [1, 1, 0], [0, 0, -1], [0, 0, 1]]
        bounds = [0, 0, 1, 0, 1]
        best = None
        for count in range(4):
            for active in itertools.combinations(range(5), count):
                matrix = [row[:] + [Decimal(constraints[j][i]) for j in active] for i, row in enumerate(hessian)]
                matrix += [[Decimal(x) for x in constraints[j]] + [Decimal(0)] * count for j in active]
                values = oracle.solve(matrix, [-x for x in linear] + [Decimal(bounds[j]) for j in active])
                if values is None:
                    continue
                z = values[:3]
                if any(g.dot(row, z) > bound + Decimal("1e-55") for row, bound in zip(constraints, bounds)):
                    continue
                beta = [max(Decimal(0), 1-z[0]-z[1]), max(Decimal(0), z[0]), max(Decimal(0), z[1])]
                total = sum(beta)
                if not total:
                    continue
                beta = [x / total for x in beta]
                s = min(Decimal(1), max(Decimal(0), z[2]))
                point = [delta[k] + beta[1]*u[k] + beta[2]*v[k] - s*axis[k] for k in range(3)]
                squared = g.dot(point, point)
                if best is None or squared < best[0]:
                    best = (squared, point, beta, s)
        if best is None:
            return {"distance_lower_m": 0., "distance_upper_m": None, "uncertain": True,
                    "method": "decimal100_no_feasible_candidate"}
        squared, point, beta, s = best
        upper = squared.sqrt()
        error = Decimal("1e-75")
        lower = Decimal(0)
        if upper > error:
            normal = [x / upper for x in point]
            # Coordinates relative to triangle0: axis a=-delta,b=axis-delta.
            lower = min(Decimal(0), g.dot(normal, u), g.dot(normal, v)) - max(-g.dot(normal, delta), g.dot(normal, g.sub(axis, delta)))
        lo = max(Decimal(0), lower-error) * scale
        hi = (upper+error) * scale
        certified = hi-lo <= Decimal("1e-50") * scale
        return {"distance_lower_m": max(0., math.nextafter(float(lo), -math.inf)),
                "distance_upper_m": math.nextafter(float(hi), math.inf), "uncertain": not certified,
                "method": "decimal100_active_sets_primal_dual", "barycentric": list(map(float, beta)), "axis_fraction": float(s)}


def distance_interval(triangle, capsule, *, allow_fallback=True):
    origin = triangle[0]
    relative = [g.sub(p, origin) for p in triangle + [capsule["a"], capsule["b"]]]
    scale = max(abs(x) for p in relative for x in p)
    if not math.isfinite(scale):
        return {"distance_lower_m": 0., "distance_upper_m": None, "uncertain": True, "method": "nonfinite_relative_geometry"}
    if not scale:
        return {"distance_lower_m": 0., "distance_upper_m": 0., "uncertain": False,
                "method": "identical_point", "barycentric": [1., 0., 0.], "axis_fraction": 0.}
    local = [[x / scale for x in p] for p in relative]
    tri, a, b = local[:3], local[3], local[4]
    _, beta, s, feature = features.triangle_axis(tri, {"a": a, "b": b})
    beta = [max(0., x) for x in beta]
    beta = [x / sum(beta) for x in beta]
    s = min(1., max(0., s))
    p, q = features.interpolate(tri, beta), g.lerp(a, b, s)
    delta = g.sub(p, q)
    upper = math.hypot(*delta)
    world_scale = max(abs(x) for p in triangle+[capsule["a"], capsule["b"]] for x in p)
    error = 4096 * EPS * max(1., world_scale/scale, *(math.hypot(*v) for v in local))
    if not math.isfinite(error):
        if allow_fallback:
            return high_precision_interval(triangle, capsule["a"], capsule["b"])
        return {"distance_lower_m": 0., "distance_upper_m": None, "uncertain": True, "method": "unbounded_f64_roundoff"}
    lower = 0.
    if upper > error:
        normal = [x / upper for x in delta]
        lower = min(g.dot(normal, v) for v in tri) - max(g.dot(normal, a), g.dot(normal, b))
    lower, upper = max(0., lower-error), upper+error
    if upper-lower > 8*error:
        if allow_fallback:
            return high_precision_interval(triangle, capsule["a"], capsule["b"])
        return {"distance_lower_m": lower*scale, "distance_upper_m": upper*scale, "uncertain": True,
                "method": "unresolved_f64_primal_dual", "barycentric": beta, "axis_fraction": s}
    return {"distance_lower_m": max(0., math.nextafter(lower*scale, -math.inf)),
            "distance_upper_m": math.nextafter(upper*scale, math.inf), "uncertain": False,
            "method": "f64_prior_features_primal_dual", "barycentric": beta, "axis_fraction": s, "feature": feature}


def aabb_proves_separation(triangle, capsule, radius):
    # Each interval contains its primitive; any separating coordinate proves separation.
    scale = max(1., radius, *(abs(x) for p in triangle+[capsule["a"], capsule["b"]] for x in p))
    margin = 4096*EPS*scale
    for j in range(3):
        low, high = min(p[j] for p in triangle), max(p[j] for p in triangle)
        axis_low, axis_high = min(capsule["a"][j], capsule["b"][j]), max(capsule["a"][j], capsule["b"][j])
        if low-axis_high > radius+margin or axis_low-high > radius+margin:
            return True
    return False


def conservative_policy_zero_certificate(meta, positions):
    """Sufficient all-pair clearance from documented1e-7 skinny/underflow policy."""
    unproved = []
    minimum_margin = math.inf
    minimum_relative_normal_squared = math.inf
    checked = 0
    for ti, indices in enumerate(meta["triangles"]):
        tri = [positions[i] for i in indices]
        for ci, capsule in enumerate(meta["project"]["mannequin"]["capsules"]):
            checked += 1
            points = tri+[capsule["a"], capsule["b"]]
            relative = [g.sub(p, tri[0]) for p in points]
            scale = max(abs(x) for p in relative for x in p)
            reason = None
            if not math.isfinite(scale) or scale == 0.:
                reason = "zero_or_nonfinite_pair_scale"
            else:
                local = [[x/scale for x in p] for p in relative]
                u, v = local[1], local[2]
                longest = max(math.hypot(*edge) for edge in (u, v, g.sub(v, u)))
                normal = math.hypot(*g.cross(u, v))
                world = max(abs(x) for p in points for x in p)
                error = 4096*EPS*max(1., world/scale)
                longest_upper = longest+8*error
                normal_lower = max(0., normal-32*error*(1+longest)**2)
                margin = normal_lower-1e-7*longest_upper**2
                minimum_margin = min(minimum_margin, margin)
                minimum_relative_normal_squared = min(minimum_relative_normal_squared, normal_lower**2)
                if longest <= 1e-125 or normal_lower <= 1e-125:
                    reason = "not_proved_clear_of_squared_underflow"
                elif margin <= 64*error:
                    reason = "not_decisively_clear_of_documented_skinny_threshold"
            if reason:
                unproved.append({"triangle_index": ti, "capsule_index": ci, "reason": reason})
    return {"agreement_scope": "Sufficient zero-count proof only; checks ALL pairs, including AABB-separated pairs",
            "documented_skinny_threshold": 1e-7, "expected_reported_conservative_pairs": 0 if not unproved else None,
            "zero_count_proved": not unproved, "checked_pairs": checked, "unproved_pairs": len(unproved), "unproved_geometry": unproved,
            "minimum_outward_normal_minus_threshold_margin": minimum_margin if math.isfinite(minimum_margin) else None,
            "minimum_outward_normal_squared": minimum_relative_normal_squared if math.isfinite(minimum_relative_normal_squared) else None}


def measure(meta, positions, *, exhaustive=False):
    pieces = {p["id"]: p for p in meta["project"]["pieces"]}
    materials = {m["id"]: m for m in meta["project"]["materials"]}
    offsets = [materials[pieces[pid]["material"]]["collision_thickness"] for pid in meta["piece_ids"]]
    maximum_lo = maximum_hi = 0.
    worst = None
    uncertain = []
    counts = {"total_pairs": 0, "aabb_zero_occupancy_pairs": 0, "distance_evaluated_pairs": 0,
              "decimal100_pairs": 0, "positive_upper_depth_pairs": 0,
              "intersection_pairs_depth_over_1e_minus8_lower": 0, "intersection_pairs_depth_over_1e_minus8_upper": 0,
              "intersection_threshold_ambiguous_pairs": 0}
    for ti, indices in enumerate(meta["triangles"]):
        triangle = [positions[i] for i in indices]
        offset = max(offsets[i] for i in indices)
        for ci, capsule in enumerate(meta["project"]["mannequin"]["capsules"]):
            counts["total_pairs"] += 1
            radius = capsule["radius"] + offset
            if not exhaustive and aabb_proves_separation(triangle, capsule, radius):
                counts["aabb_zero_occupancy_pairs"] += 1
                continue
            interval = distance_interval(triangle, capsule)
            counts["distance_evaluated_pairs"] += 1
            counts["decimal100_pairs"] += interval["method"].startswith("decimal100")
            lower = max(0., radius-interval["distance_upper_m"]) if interval["distance_upper_m"] is not None else 0.
            upper = max(0., radius-interval["distance_lower_m"])
            maximum_lo = max(maximum_lo, lower)
            if upper > maximum_hi:
                maximum_hi = upper
                worst = {"triangle_index": ti, "capsule_index": ci, "piece_id": meta["piece_ids"][indices[0]],
                         "vertices_0based": indices, "physical_offset_m": offset,
                         "penetration_lower_m": lower, "penetration_upper_m": upper, **interval}
            counts["positive_upper_depth_pairs"] += upper > 0.
            counts["intersection_pairs_depth_over_1e_minus8_lower"] += lower > 1e-8
            counts["intersection_pairs_depth_over_1e_minus8_upper"] += upper > 1e-8
            counts["intersection_threshold_ambiguous_pairs"] += lower <= 1e-8 < upper
            if interval["uncertain"]:
                uncertain.append({"triangle_index": ti, "capsule_index": ci, **interval})
    return {"penetration_lower_m": maximum_lo, "penetration_upper_m": maximum_hi, "worst": worst,
            "uncertain_pairs": len(uncertain), "uncertain_geometry": uncertain, "counts": counts,
            "scope": "Full current static triangle/capsule surface; max incident vertex physical offset; no swept/CCD claim"}
