"""Independently balance each saved reduced root and audit traction on the spherical shoulder."""

import json
import math
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
BALANCE = HERE.with_name("harbor-shoulder-balance-20261122")
sys.path.insert(0, str(BALANCE))
from check_balance import (  # noqa: E402 (frozen adjacent evidence module)
    bend,
    capsule_distance,
    cross,
    dot,
    material_vectors,
    normalized,
    sub,
)


def traction(contact, x, data, coefficient):
    w, ids = contact["weights"], contact["vertices"]

    assert len(ids) == len(w) == 3 and min(w) >= 0 and abs(sum(w) - 1.0) < 1e-12

    def point(positions):
        return [
            sum(weight * positions[i][k] for weight, i in zip(w, ids)) for k in range(3)
        ]

    now, last = point(x), point(data["previous"])
    cap = data["capsules"][contact["body"]]
    axis = sub(cap[3:6], cap[:3])
    # All of the loaded material points project onto this capsule's upper spherical cap.
    assert min(dot(sub(p, cap[3:6]), axis) for p in (now, last)) > 0
    end, start = (normalized(sub(p, cap[3:6])) for p in (now, last))
    radius = cap[6] + data["material"]["collision_thickness"]
    assert abs(math.dist(now, cap[3:6]) - radius) < 1e-9
    arc_angle = math.atan2(math.hypot(*cross(start, end)), dot(start, end))
    # The arriving tangent of the great-circle arc from start to end.
    direction = normalized(cross(cross(start, end), end))
    force = contact["force"]
    pressure = dot(force, end)
    tangent = sub(force, [pressure * v for v in end])
    error = math.hypot(
        *(f + coefficient * pressure * t for f, t in zip(tangent, direction))
    )
    assert pressure > 0 and error < 1e-6 and arc_angle > 1e-7
    slip = sub(now, last)
    projected = sub(slip, [dot(slip, end) * v for v in end])
    return dict(
        normal_N=pressure,
        tangential_N=math.hypot(*tangent),
        arc_slip_mm=1000 * radius * arc_angle,
        projected_slip_mm=1000 * math.hypot(*projected),
        geodesic_force_error_N=error,
        friction_work_J=dot(tangent, slip),
    )


def measure(data, row, mode):
    nodes = row["mobile"]
    assert nodes == data["cases"]["ring"]["mobile"] and len(nodes) == 14
    assert len(row["positions"]) == len(nodes)
    x = [p.copy() for p in data["current"]]
    for i, p in zip(nodes, row["positions"]):
        x[i] = p.copy()

    def positions(indices):
        return [x[i] for i in indices]

    faces = [
        (i, t, [data["uv"][j] for j in t])
        for i, t in zip(data["face_ids"], data["triangles"])
        if set(t) & set(nodes)
    ]
    hinges = [
        (h, *bend([data["rest"][i] for i in h]))
        for h in data["hinges"]
        if set(h) & set(nodes)
    ]
    material = data["material"]
    assert abs(material["grain_angle"] - math.pi / 2) < 1e-12

    def energy():
        terms = []
        for _, tri, uv in faces:
            warp, weft, area = material_vectors(positions(tri), uv)
            for modulus, strain in [
                (material["warp_stiffness"], (dot(warp, warp) - 1) / 2),
                (material["weft_stiffness"], (dot(weft, weft) - 1) / 2),
                (material["shear_stiffness"], dot(warp, weft)),
            ]:
                terms.append(area * modulus * strain**2 / 2)
        for hinge, rest, scale in hinges:
            angle = bend(positions(hinge))[0] - rest
            terms.append(
                material["bending_stiffness"]
                * scale
                * math.atan2(math.sin(angle), math.cos(angle)) ** 2
                / 2
            )
        return math.fsum(terms)

    forces = {
        i: [
            data["masses"][i] * (a - b) / data["dt"] ** 2
            for a, b in zip(x[i], data["predicted"][i])
        ]
        for i in nodes
    }
    for i in nodes:
        for k in range(3):
            value = x[i][k]
            samples = []
            h = 2e-6
            for offset in (-2, -1, 1, 2):
                x[i][k] = value + offset * h
                samples.append(energy())
            x[i][k] = value
            a, b, c, d = samples
            forces[i][k] += (a - 8 * b + 8 * c - d) / (12 * h)
    tension = []
    for seam in data["sews"]:
        weights = seam["coefficients"]
        delta = [sum(w * x[i][k] for i, w in weights) for k in range(3)]
        t = (math.hypot(*delta) - seam["goal_m"]) / seam["compliance"]
        tension.append(t)
        for i, w in weights:
            if i in forces:
                for k, v in enumerate(normalized(delta)):
                    forces[i][k] += w * t * v
    if "tensions" in row:
        assert max(abs(a - b) for a, b in zip(tension, row["tensions"])) < 1e-5
    contacts = row["contact"]
    assert bool(contacts) == (mode != "released")
    records = [
        traction(c, x, data, material["friction"] if mode == "recorded" else 0)
        for c in contacts
    ]
    for c in contacts:
        for i, w in zip(c["vertices"], c["weights"]):
            if i in forces:
                for k, v in enumerate(c["force"]):
                    forces[i][k] -= w * v
    residual = math.hypot(*(v for force in forces.values() for v in force))
    assert residual < 1e-6
    clearance = min(
        capsule_distance(positions(t), cap) - cap[6] - material["collision_thickness"]
        for _, t, _ in faces
        for cap in data["capsules"]
    )
    assert mode == "released" or clearance >= -1e-9
    face = next(v for v in faces if v[0] == 6)
    stretch = math.hypot(*material_vectors(positions(face[1]), face[2])[0])
    reference = data["cases"]["ring"]["positions"]
    max_motion = max(math.dist(a, b) for a, b in zip(positions(nodes), reference))
    assert (
        row.get("native_swept_self_contact_count", row.get("swept_self_contact_count"))
        == 0
    )
    return dict(
        front_warp_strain=stretch - 1,
        front_material_line_mm=50 * stretch,
        min_original_capsule_gap_mm=1000 * clearance,
        max_motion_from_recorded_root_mm=1000 * max_motion,
        shoulder_tension_N=tension[0],
        mobile_momentum_residual_N=residual,
        tangential_work_J=sum(q["friction_work_J"] for q in records),
        loaded_contacts=records,
    )


if __name__ == "__main__":
    results = {}
    for case in ("original", "extended"):
        data = json.loads((BALANCE / (case + ".json")).read_text())
        rows = json.loads((HERE / (case + ".json")).read_text())
        rows = {"recorded": data["cases"]["ring"], **rows}
        results[case] = {mode: measure(data, row, mode) for mode, row in rows.items()}
    print(json.dumps(results, indent=2, allow_nan=False))
