"""Independently evaluate a native strain display from physical OBJ edges and PNG pixels.

Run with an evidence directory, the original rest OBJ, and the original two-fabric project.
Geometry uses Python's standard library. Image comparisons use Pillow.
"""

import argparse
import collections
import json
import math
from pathlib import Path
from PIL import Image, ImageDraw


def read_obj(path):
    vertices, uv, faces, mtls = [], [], [], []
    group = material = None
    for line in path.read_text().splitlines():
        parts = line.split()
        if not parts:
            continue
        op, args = parts[0], parts[1:]
        if op == "v":
            vertices.append(tuple(map(float, args)))
        elif op == "vt":
            uv.append(tuple(map(float, args)))
        elif op == "g":
            (group,) = args
        elif op == "usemtl":
            (material,) = args
        elif op == "mtllib":
            mtls.extend(args)
        elif op == "f":
            assert group and material and len(args) == 3
            indices = [tuple(int(v) - 1 for v in tok.split("/")) for tok in args]
            assert all(i >= 0 and j >= 0 and k >= 0 for i, j, k in indices)
            faces.append(
                (
                    group,
                    material,
                    tuple(i for i, j, k in indices),
                    tuple(j for i, j, k in indices),
                )
            )
    assert len(mtls) == 1 and len(set(f[1] for f in faces)) == 2
    return vertices, uv, faces, mtls[0]


def dot(a, b):
    return sum(v * w for v, w in zip(a, b))


def sub(a, b):
    return tuple(v - w for v, w in zip(a, b))


def stretch(uv, xyz):
    a, b = sub(uv[1], uv[0]), sub(uv[2], uv[0])
    p, q = sub(xyz[1], xyz[0]), sub(xyz[2], xyz[0])
    aa, ab, bb = dot(a, a), dot(a, b), dot(b, b)
    pp, pq, qq = dot(p, p), dot(p, q), dot(q, q)
    rest_det = (a[0] * b[1] - a[1] * b[0]) ** 2
    # Roots of det(G_current - lambda * G_rest) = 0 are squared principal stretches.
    trace = (bb * pp + aa * qq - 2 * ab * pq) / rest_det
    cross = (
        p[1] * q[2] - p[2] * q[1],
        p[2] * q[0] - p[0] * q[2],
        p[0] * q[1] - p[1] * q[0],
    )
    determinant = dot(cross, cross) / rest_det
    return math.sqrt(
        (trace + math.sqrt(max(0.0, trace * trace - 4 * determinant))) / 2.0
    )


def palette(ratio, upper):
    colors = [(45, 105, 163), (63, 171, 153), (238, 193, 92), (215, 77, 62)]
    level = (
        min(max((ratio - 1.0) / (upper / 100 - 1.0), 0.0), 1.0)
        if upper > 100
        else float(ratio > 1.0 + 1e-12)
    )
    x = 3 * level
    i = min(int(x), 2)
    return tuple(
        int(a + (b - a) * (x - i) + 0.5) for a, b in zip(colors[i], colors[i + 1])
    )


def rgb(path):
    with Image.open(path) as im:
        assert im.size == (1640, 1000)
        return im.convert("RGB")


def check(folder, rest_obj, original):
    base_vertices, base_uv, base_faces, base_mtl = read_obj(rest_obj)
    vertices, uv, faces, mtl = read_obj(folder / "draped.obj")
    assert (
        vertices != base_vertices
        and uv == base_uv
        and faces == base_faces
        and mtl == base_mtl
    )
    assert (folder / mtl).read_bytes() == (rest_obj.parent / mtl).read_bytes()
    assert len(vertices) == 654 and len(faces) == 1172
    project = json.loads(original.read_bytes())
    assert project == json.loads((folder / "reopened.weftforge").read_bytes())
    ratios = [
        stretch([uv[i] for i in u], [vertices[i] for i in v]) for _, _, v, u in faces
    ]
    peaks = {
        group: max(s for f, s in zip(faces, ratios) if f[0] == group)
        for group in sorted({f[0] for f in faces})
    }
    observed = json.loads((folder / "NATIVE.json").read_bytes())
    assert abs(max(ratios) * 100 - observed["draped_peak_percent"]) < 0.005
    before = rgb(folder / "map-off.png")
    enabled = rgb(folder / "map-on.png")
    after = rgb(folder / "map-off-repeat.png")
    repeat = rgb(folder / "map-on-repeat.png")
    wider = rgb(folder / "map-125.png")
    assert before.tobytes() == after.tobytes() and enabled.tobytes() == repeat.tobytes()
    assert abs(peaks["piece_2"] * 100 - observed["isolated_front_peak_percent"]) < 0.005
    area = (47, 145, 695, 943)
    pixels = lambda im, box: im.crop(box).tobytes()
    flat = pixels(before, area)
    assert all(pixels(im, area) == flat for im in (enabled, after, repeat, wider))
    fabric_sample = before.crop(area)
    fabric_counts = collections.Counter(
        fabric_sample.getpixel((x, y))
        for y in range(fabric_sample.height)
        for x in range(fabric_sample.width)
    )
    materials = project["project"]["materials"]
    swatches = {
        m["name"]: fabric_counts[tuple(int(255 * c + 0.5) for c in m["color"])]
        for m in materials
    }
    assert min(swatches.values()) > 1000
    # Camera calibration uses the exported native View > Front and Frame garment,
    # plus the observed canvas bounds in the native PNG. No studio code runs here.
    left, top, right, bottom = observed["canvas_bounds_px"]
    low = [min(p[i] for p in vertices) for i in range(2)]
    high = [max(p[i] for p in vertices) for i in range(2)]
    scale = min(
        1500.0,
        (right - left - 56) / (high[0] - low[0]),
        (bottom - top - 56) / (high[1] - low[1]),
    )
    projected = [
        (
            (left + right) / 2 + scale * (x - (low[0] + high[0]) / 2),
            (top + bottom) / 2 - scale * (y - (low[1] + high[1]) / 2),
        )
        for x, y, z in vertices
    ]
    # Painter order uses the frontmost mean Z; ignore edges and sample triangle interiors.
    face_img = Image.new("I", before.size, 0)
    draw = ImageDraw.Draw(face_img)
    for idx in sorted(
        range(len(faces)), key=lambda i: sum(vertices[j][2] for j in faces[i][2])
    ):
        draw.polygon([projected[j] for j in faces[idx][2]], fill=idx + 1)
    hits = mapped = uncovered = repeat_off = repeat_on = changes = wider_changes = 0
    unique = set()
    for y in range(math.ceil(top) + 2, math.floor(bottom) - 2, 3):
        for x in range(math.ceil(left) + 2, math.floor(right) - 2, 3):
            if any(
                a <= x <= c and b <= y <= d
                for a, b, c, d in observed["excluded_overlay_rects_px"]
            ):
                continue
            pix = (x, y)
            index = face_img.getpixel(pix) - 1
            if index < 0:
                continue
            if any(
                face_img.getpixel((x + dx, y + dy)) != index + 1
                for dx, dy in [(2, 0), (-2, 0), (0, 2), (0, -2)]
            ):
                continue
            off, on, again, on_again, wide = [
                im.getpixel(pix) for im in (before, enabled, after, repeat, wider)
            ]
            match = lambda a, b: max(abs(k - l) for k, l in zip(a, b)) <= 1
            expected = palette(ratios[index], 120)
            expected_wider = palette(ratios[index], 125)
            hits += match(on, expected)
            mapped += match(wide, expected_wider)
            repeat_off += off == again
            repeat_on += on == on_again
            changes += off != on
            wider_changes += on != wide
            unique.add(index)
            uncovered += 1
    assert uncovered > 1000 and len(unique) > 80, (uncovered, len(unique))
    assert min(hits, mapped) / uncovered > 0.995, (hits, mapped, uncovered)
    assert (
        repeat_off == repeat_on == uncovered
        and changes > 0.98 * uncovered
        and wider_changes > 50
    )
    cutout = rgb(folder / "cutout.png")
    # Probe the sloped left cutout boundary in the selected Pocket capture.
    # Each sampled row starts on fabric and crosses the selection outline into the opening.
    fabric = (56, 140, 194)
    outline = (92, 163, 225)
    background = (37, 37, 37)
    rows = 0
    mixed = set()
    for y in range(430, 651):
        band = [cutout.getpixel((x, y)) for x in range(225, 310)]
        assert band[0] == fabric and band[-1] in (
            background,
            (44, 44, 44),
            (45, 45, 45),
        )
        if band[-1] != background:
            continue  # The one horizontal grid line crossing this calibrated strip.
        start = next(i for i, pixel in enumerate(band) if pixel != fabric)
        end = next(
            i for i, pixel in enumerate(band[start:], start) if pixel == background
        )
        edge = band[start:end]
        assert outline in edge and end - start <= 9, (y, edge)
        intermediate = {
            pixel for pixel in edge if pixel not in (fabric, outline, background)
        }
        assert intermediate
        mixed.update(intermediate)
        rows += 1
    interior = [
        cutout.getpixel((x, y)) for y in range(440, 490) for x in range(365, 410)
    ]
    assert all(r == g == b and 37 <= r <= 45 for r, g, b in interior)
    return {
        "cutout_open_interior_grid_pixels": len(interior),
        "cutout_antialiased_boundary_rows": rows,
        "cutout_distinct_intermediate_edge_colors": len(mixed),
        "identical_full_workspace_pixels_on_repeat": before.width * before.height,
        "triangles": len(faces),
        "vertices": len(vertices),
        "revision": project["project"]["revision"],
        "rest_topology_uv_material_library_preserved": True,
        "document_preserved": True,
        "range_computed_from_export_percent": [min(ratios) * 100, max(ratios) * 100],
        "per_piece_max_percent": {g: v * 100 for g, v in peaks.items()},
        "material_colored_2d_pixels": swatches,
        "identical_2d_canvas_pixels": (area[2] - area[0]) * (area[3] - area[1]),
        "sampled_visible_faces": len(unique),
        "sampled_face_interior_pixels": uncovered,
        "matching_global_120_pixels": hits,
        "matching_global_125_pixels": mapped,
        "off_to_on_changed_pixels": changes,
        "120_to_125_changed_pixels": wider_changes,
        "off_repeat_identical_pixels": repeat_off,
        "on_repeat_identical_pixels": repeat_on,
    }


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    for arg in ("folder", "rest_obj", "original"):
        parser.add_argument(arg, type=Path)
    args = parser.parse_args()
    print(json.dumps(check(args.folder, args.rest_obj, args.original), indent=2))
