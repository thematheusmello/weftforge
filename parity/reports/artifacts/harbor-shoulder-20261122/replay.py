"""Measure both shoulder panels from six vertices and replay each solver stage."""
import json
import lzma
import math
from collections import defaultdict
from pathlib import Path

HERE = Path(__file__).resolve().parent
STAGES = ["material", "bend", "soft_sewing", "hard_sewing", "avatar_ground", "self_contact", "pins"]
ORDER = [(-1, "previous"), (-1, "predict")] + [(i, stage) for i in range(32) for stage in STAGES]


def endpoint_weight(uv):
    (a, b, c) = uv
    assert b[1] == c[1] and abs(a[1] - b[1] - 0.05) < 1e-12
    fraction = (a[0] - b[0]) / (c[0] - b[0])
    assert 0 < fraction < 1
    return fraction


def endpoint(triangle, weight):
    return [b + weight * (c - b) for b, c in zip(triangle[1], triangle[2])]


def length(triangle, weight):
    return math.dist(triangle[0], endpoint(triangle, weight))


def frame28_witness(case, panels, uv, weights):
    project_path = HERE.parent / "native-hem-regeneration-20261120" / "full-fit" / case / "Harbor.weftforge"
    project = json.loads(project_path.read_text())["project"]
    capsules = project["mannequin"]["capsules"]
    assert len(capsules) == 6
    thickness, = {m["collision_thickness"] for m in project["materials"]}
    moved = []
    clearances = []
    strains = []
    for side, (panel, chart, weight) in enumerate(zip(panels, uv, weights)):
        top = [panel[0][0] - 0.008, panel[0][1] - 0.005, panel[0][2]]
        triangle = [top, *panel[1:]]
        moved.append(top)
        strains.append([length(triangle, weight) / 0.05 - 1,
                        math.dist(*triangle[1:]) / math.dist(*chart[1:]) - 1])
        # A separating plane bounds clearance for the entire triangle and capsule.
        z = 0.20 if case == "original" else (0.18, 0.21)[side]
        sign = (1, -1)[side]
        normals = [(-0.605, 0.771, sign * z), (-0.975, 0, sign * 0.22)] + [(0, 1, 0)] * 4
        for capsule, normal in zip(capsules, normals):
            def projection(point):
                return sum(x*n for x, n in zip(point, normal)) / math.hypot(*normal)
            gap = min(map(projection, triangle)) - max(map(projection, (capsule["a"], capsule["b"])))
            clearances.append(gap - capsule["radius"] - thickness)
    assert all(abs(e) < 0.15 for pair in strains for e in pair)
    assert min(clearances) > 0
    assert abs(math.dist(*moved) - math.dist(*[t[0] for t in panels])) < 1e-12
    return {"front_back_warp_and_weft_strain": strains,
            "minimum_certified_capsule_clearance_mm": min(clearances) * 1000}


def read_case(case):
    header, *rows = map(json.loads, (HERE / (case + ".jsonl")).read_text().splitlines())
    assert header["seam"] == 64 and header["faces"][0] == 6
    assert all(abs(a - math.pi / 2) < 1e-12 for a in header["grain"])
    weights = [endpoint_weight(uv) for uv in header["uv"]]
    poses = {(r["branch"], r["frame"]): r for r in rows}
    captures = defaultdict(list)
    branch = "canonical"
    with lzma.open(HERE / (case + ".trace.xz"), "rt") as trace:
        for line in trace:
            if line.startswith("BRANCH "):
                branch = line.split()[1]
                continue
            label, frame, substep, iteration, stage, values = line.split(maxsplit=5)
            assert label == "TRACE"
            captures[branch, int(frame)].append((int(substep), int(iteration), stage, json.loads(values)))
    assert list(captures) == [("canonical", 27), ("hold", 28), ("canonical", 28), ("canonical", 29)]
    result = {}
    for (branch, frame), samples in captures.items():
        state = poses[branch, frame]
        front, back = state["positions"]
        steps = defaultdict(list)
        previous = poses["canonical", frame - 1]["positions"][0]
        initial_length = length(previous, weights[0])
        assert [(s, i, k) for s, i, k, _ in samples] == [
            (s, i, k) for s in range(8) for i, k in ORDER
        ]
        for _, _, stage, x in samples:
            if stage == "previous":
                assert previous == x
            else:
                steps[stage].append(length(x, weights[0]) - length(previous, weights[0]))
            previous = x
        assert previous == front
        front_length = length(front, weights[0])
        totals = {stage: 1000 * math.fsum(deltas) for stage, deltas in steps.items()}
        assert abs(math.fsum(totals.values()) - 1000 * (front_length - initial_length)) < 1e-9
        anchors = [endpoint(p, w) for p, w in zip((front, back), weights)]
        span = math.dist(*anchors)
        gap = math.dist(front[0], back[0])
        # The optimal free endpoints lie on the line between the fixed anchors.
        minimum_length = (span - gap) / 2
        assert minimum_length > 0
        opt = [[a + (b - a) * t for a, b in zip(*anchors)]
               for t in (minimum_length / span, 1 - minimum_length / span)]
        assert abs(math.dist(*opt) - gap) < 1e-12
        assert all(abs(math.dist(a, b) - minimum_length) < 1e-12 for a, b in zip(opt, anchors))
        back_length = length(back, weights[1])
        assert max(front_length, back_length) >= minimum_length - 1e-12
        peak = state["diagnostics"]["peak_strain"]
        if peak["triangle"] in header["faces"] and peak["axis"] == "warp":
            observed = (front_length, back_length)[header["faces"].index(peak["triangle"])]
            assert abs(observed / 0.05 - 1 - peak["signed_strain"]) < 1e-12
        result[f"{branch}-{frame}"] = {
            "front_mm": front_length * 1000, "back_mm": back_length * 1000,
            "stitch_gap_mm": gap * 1000, "anchor_span_mm": span * 1000,
            "minimum_possible_max_line_mm_at_fixed_anchors_and_gap": minimum_length * 1000,
            "signed_front_length_changes_mm": totals,
        }
    assert result["canonical-28"]["front_mm"] > 57.5 > result["hold-28"]["front_mm"]
    bound = "minimum_possible_max_line_mm_at_fixed_anchors_and_gap"
    assert result["canonical-29"][bound] > 57.5 > result["canonical-28"][bound]
    result["frame28_rigid_stitch_translation"] = frame28_witness(
        case, poses["canonical", 28]["positions"], header["uv"], weights)
    return result


if __name__ == "__main__":
    print(json.dumps({case: read_case(case) for case in ("original", "extended")}, indent=2))
