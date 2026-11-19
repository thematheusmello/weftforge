"""Read-only vector and Poppler verification for the frozen original patterns."""
import argparse
from hashlib import sha256
from itertools import pairwise
import json
import math
from pathlib import Path
import re
import struct
import subprocess

import pypdf
from pypdf import PdfReader
from pypdf.generic import ContentStream

ROOT = Path(__file__).resolve().parent
SPEC_SHA = "c0986ae308055f483746f8dc8ab82579fd81805749d1207942439eac19ceec6f"
PT = 72 / 25.4
TOL = 0.0001
IDENTITY = (1, 0, 0, 1, 0, 0)
TEXT_OPS = {b"BT", b"Tf", b"Td", b"Tj", b"ET"}


def digest(raw):
    return sha256(raw).hexdigest()


def near(actual, expected, label, tolerance=TOL):
    actual, expected = tuple(map(float, actual)), tuple(map(float, expected))
    assert len(actual) == len(expected), (label, actual, expected)
    assert all(math.isclose(a, b, abs_tol=tolerance, rel_tol=0)
               for a, b in zip(actual, expected)), (label, actual, expected)


def in_points(values):
    return tuple(float(value) * PT for value in values)


def curve_extrema(start, controls, end):
    if controls is None:
        return (start, end)
    knots = (start, *controls, end)
    parameters = {0.0, 1.0}
    for x0, x1, x2, x3 in zip(*knots):
        a = 3 * (-x0 + 3*x1 - 3*x2 + x3)
        b = 6 * (x0 - 2*x1 + x2)
        c = 3 * (x1 - x0)
        if abs(a) < 1e-12:
            roots = (-c / b,) if abs(b) >= 1e-12 else ()
        else:
            discriminant = b*b - 4*a*c
            roots = ((-b - math.sqrt(discriminant)) / (2*a),
                     (-b + math.sqrt(discriminant)) / (2*a)) if discriminant >= 0 else ()
        parameters.update(root for root in roots if 0 < root < 1)
    samples = []
    for t in sorted(parameters):
        current = knots
        for _ in range(3):
            current = tuple(tuple((1-t)*a + t*b for a, b in zip(p, q))
                            for p, q in pairwise(current))
        samples.append(current[0])
    return samples


def bounds(points):
    x, y = zip(*points)
    return ((min(x), min(y)), (max(x), max(y)))


def fixture_arts(project, frozen):
    points = {point["id"]: tuple(1000*x for x in point["position"])
              for point in project["points"]}
    curves = {item["id"]: item for item in project["curves"]}

    def edge(curve_id):
        curve = curves[curve_id]
        controls = curve["controls"]
        return (points[curve["start"]],
                None if controls is None else [tuple(1000*x for x in p) for p in controls],
                points[curve["end"]])

    for piece, plan in zip(project["pieces"], frozen, strict=True):
        assert (piece["id"], piece["name"]) == (plan["id"], plan["name"])
        exterior = [point for curve_id in piece["boundary"] for point in curve_extrema(*edge(curve_id))]
        for actual, wanted in zip(bounds(exterior), plan["exterior_mm"]):
            near(actual, wanted, "independent analytic exterior")
        marks = []
        for notch in piece["notches"]:
            start, controls, end = edge(notch["curve"])
            assert controls is None, "fixture notch lies on a line"
            dx, dy = (b - a for a, b in zip(start, end))
            length = math.hypot(dx, dy)
            center = tuple(a + notch["t"] * (b-a) for a, b in zip(start, end))
            half = 500 * notch["length"]
            normal = (-dy / length * half, dx / length * half)
            marks.append(tuple(tuple(a + sign*b for a, b in zip(center, normal)) for sign in (-1, 1)))
        mark_points = [p for mark in marks for p in mark]
        for actual, wanted in zip(bounds(exterior + mark_points), plan["printed_art_mm"]):
            near(actual, wanted, "ink geometry before margin")
        origin = plan["form_origin_mm"]

        def local(point):
            return in_points(p - o for p, o in zip(point, origin))

        paths = []
        for loop in [piece["boundary"], *(hole["boundary"] for hole in piece["holes"])]:
            initial = edge(loop[0])[0]
            path = [(b"m", local(initial))]
            tail = initial
            for curve_id in loop:
                start, controls, end = edge(curve_id)
                assert start == tail, "ordered authored contour"
                if controls:
                    path.append((b"c", (*local(controls[0]), *local(controls[1]), *local(end))))
                    near((*controls[0], *controls[1]),
                         sum(map(tuple, plan["cubic_controls_mm"]), ()), "authored cubic controls")
                else:
                    path.append((b"l", local(end)))
                tail = end
            assert tail == initial
            if loop != piece["boundary"]:
                low, high = bounds([edge(curve_id)[0] for curve_id in loop])
                near((b-a for a, b in zip(low, high)), plan["hole_size_mm"], "hole")
            paths.append((False, [*path, (b"h", ())]))
        for start, end in marks:
            near((math.dist(start, end),), (plan["notch_length_mm"],), "authored notch length")
            paths.append((False, [(b"m", local(start)), (b"l", local(end))]))
        for curve_id in piece["internal_curves"]:
            start, controls, end = edge(curve_id)
            assert controls is None
            near((math.dist(start, end),), (plan["internal_length_mm"],), "authored annotation length")
            paths.append((True, [(b"m", local(start)), (b"l", local(end))]))
        yield paths


def strokes(operations, allowed=()):
    paths, pending, dashed = [], [], False
    for args, op in operations:
        if op in {b"m", b"l", b"c", b"h", b"re"}:
            pending.append((op, args))
        elif op == b"S":
            assert pending, "stroke without path"
            paths.append((dashed, pending))
            pending = []
        elif op == b"d":
            assert b"d" in allowed and args[0] and all(float(x) > 0 for x in args[0])
            dashed = True
        else:
            assert op in allowed, ("unexpected vector operator", op)
    assert not pending, "unfinished path"
    return paths


def check_paths(actual, expected, label):
    assert len(actual) == len(expected), (label, "stroke count", len(actual), len(expected))
    for (dash, path), (wanted_dash, wanted) in zip(actual, expected):
        assert dash == wanted_dash and len(path) == len(wanted), label
        for (op, coords), (wanted_op, wanted_coords) in zip(path, wanted):
            assert op == wanted_op, (label, "operator order", op, wanted_op)
            near(coords, wanted_coords, label)


def crosses(width, height, row, column, rows, columns):
    x0, x1, y0, y1 = 10, width-10, 20, height-20
    points = []
    if column:
        points += [(x0+5, y1+3), (x0+5, y0-3)]
    if column+1 < columns:
        points += [(x1-5, y1+3), (x1-5, y0-3)]
    if row:
        points += [(x0-3, y1-5), (x1+3, y1-5)]
    if row+1 < rows:
        points += [(x0-3, y0+5), (x1+3, y0+5)]
    return [
        (False, [(b"m", in_points((x-1.5, y))), (b"l", in_points((x+1.5, y))),
                 (b"m", in_points((x, y-1.5))), (b"l", in_points((x, y+1.5)))])
        for x, y in points
    ]


def check_neighbours(tiles, rows, columns):
    for row, column in tiles:
        here_box, here_marks = tiles[row, column]
        for other in ((row, column+1), (row+1, column)):
            if other not in tiles:
                continue
            other_box, other_marks = tiles[other]
            axis = 0 if other[0] == row else 1
            overlap = min(here_box[axis+2], other_box[axis+2]) - max(here_box[axis], other_box[axis])
            near((overlap,), (10*PT,), "actual clips overlap")
            coincident = sum(
                any(math.dist(mark, other_mark) <= TOL for other_mark in other_marks)
                for mark in here_marks
            )
            assert coincident == 2, (row, column, other, "matching crosses", coincident)
    assert len(tiles) == rows * columns


def normalized(text):
    return " ".join(text.split())


def read_pdf(path, project, spec, paper):
    reader = PdfReader(path, strict=True)
    width, height = spec["paper_mm"][paper]
    assert len(reader.pages) == spec["pages"]
    assert reader.root_object["/ViewerPreferences"]["/PrintScaling"] == "/None"
    assert [(entry.title, reader.get_destination_page_number(entry)) for entry in reader.outline] == [
        (entry["name"], entry["outline_start_index"]) for entry in spec["pieces"]
    ]
    for page in reader.pages:
        assert page.rotation == 0 and page.user_unit == 1
        for box in (page.mediabox, page.cropbox, page.trimbox, page.bleedbox, page.artbox):
            near(box, (0, 0, *in_points((width, height))), "physical page box")
        assert not page.get("/Annots"), "unexpected annotation overlay"

    calibration = reader.pages[0]
    assert "/XObject" not in calibration["/Resources"]
    square = in_points(((width-100)/2, (height-100)/2, 100, 100))
    check_paths(strokes(calibration.get_contents().operations, {b"w", *TEXT_OPS}),
                [(False, [(b"re", square)])], "100 mm calibration square")
    labels = ["WeftForge pattern printing", "Actual size (100%)", "Turn off Fit to page",
              "Each side should be 100 mm."]
    assert all(label in normalized(calibration.extract_text()) for label in labels)

    forms, receipts = set(), []
    for plan, paths in zip(spec["pieces"], fixture_arts(project, spec["pieces"]), strict=True):
        rows, columns = plan["rows"], plan["columns"]
        piece_id, first_page = plan["id"], plan["outline_start_index"]
        clip_size = (width-20, height-40)
        step = tuple(length-10 for length in clip_size)
        for extent, count, window, increment in zip(plan["form_extent_mm"], (columns, rows), clip_size, step):
            assert extent <= window + (count-1)*increment
            if count > 1:
                assert extent > window + (count-2)*increment
        tiles, form_id = {}, None
        for row in range(rows):
            for column in range(columns):
                index = first_page + row*columns + column
                page = reader.pages[index]
                operations = page.get_contents().operations
                assert [op for _, op in operations[:7]] == [b"q", b"re", b"W", b"n", b"cm", b"Do", b"Q"]
                x, y, w, h = map(float, operations[1][0])
                near((x, y, w, h), in_points((10, 20, *clip_size)), "clip and margins")
                expected_translation = in_points((10-column*step[0], height-20+row*step[1]))
                matrix = tuple(map(float, operations[4][0]))
                near(matrix, (*IDENTITY[:4], *expected_translation), "unit scale and row/column translation")
                tx, ty = matrix[4:]
                name, = operations[5][0]
                refs = page["/Resources"]["/XObject"]
                assert list(refs.keys()) == [name]
                form = refs[name]
                current_form = form.indirect_reference.idnum
                if form_id is None:
                    form_id = current_form
                    assert form_id not in forms, "piece must have its own art"
                    forms.add(form_id)
                    assert form["/Subtype"] == "/Form" and form.get("/FormType", 1) == 1
                    near(form.get("/Matrix", IDENTITY), IDENTITY, "unit Form matrix")
                    art_w, art_h = in_points(plan["form_extent_mm"])
                    near(form["/BBox"], (0, -art_h, art_w, 0), "analytic art bounding box")
                    assert not form["/Resources"], "unexpected nested resource"
                    check_paths(strokes(ContentStream(form, reader).operations, {b"w", b"j", b"J", b"d"}),
                                paths, f"pattern #{piece_id} authored vectors")
                assert current_form == form_id
                title = f"Pattern #{piece_id} Row {row+1}/{rows} Column {column+1}/{columns}"
                labels.append(title)
                text = normalized(page.extract_text())
                assert title in text and "Actual size (100%)" in text
                actual = strokes(operations[7:], {b"w", *TEXT_OPS})
                check_paths(actual, crosses(width, height, row, column, rows, columns), "registration vectors")
                centers = []
                for _, path in actual:
                    _, a = path[0]
                    _, b = path[1]
                    centers.append(((float(a[0])+float(b[0]))/2-tx, float(a[1])-ty))
                clip = (x-tx, y-ty, x+w-tx, y+h-ty)
                tiles[row, column] = (clip, centers)
                receipts.append({"page": index+1, "piece_id": piece_id, "row": row+1, "column": column+1,
                                 "translation_pt": [tx, ty], "registration_crosses": len(centers)})
        check_neighbours(tiles, rows, columns)
    return (receipts, labels)


def poppler(args):
    result = subprocess.run(args, capture_output=True, text=True, check=True)
    assert not result.stderr.strip(), (args[0], result.stderr)
    return result.stdout


def read_with_poppler(path, paper, spec, labels, render_dir):
    width, height = spec["paper_mm"][paper]
    pages = spec["pages"]
    info = poppler(["pdfinfo", "-f", "1", "-l", str(pages), "-box", str(path)])
    assert re.search(rf"^Pages:\s+{pages}\s*$", info, re.M)
    boxes = re.findall(r"^Page\s+(\d+)\s+MediaBox:\s+(\S+)\s+(\S+)\s+(\S+)\s+(\S+)\s*$", info, re.M)
    assert [int(box[0]) for box in boxes] == list(range(1, pages+1)), boxes
    for _, *box in boxes:
        near(box, (0, 0, *in_points((width, height))), "Poppler MediaBox", spec["poppler_box_abs_pt_tolerance"])
    poppler_text = normalized(poppler(["pdftotext", "-layout", str(path), "-"]))
    assert all(label in poppler_text for label in labels), "Poppler text validation"
    images = []
    if render_dir is not None:
        render_dir.mkdir(parents=True, exist_ok=True)
        prefix = render_dir / f"{paper}-page"
        assert not list(render_dir.glob(f"{prefix.name}-*.png")), "choose a fresh render destination"
        poppler(["pdftoppm", "-r", "72", "-png", str(path), str(prefix)])
        for number in range(1, pages+1):
            image = render_dir / f"{prefix.name}-{number}.png"
            raw = image.read_bytes()
            assert raw.startswith(b"\x89PNG\r\n\x1a\n")
            pixel_size = struct.unpack(">II", raw[16:24])
            near(pixel_size, in_points((width, height)), "72 dpi raster size", 1)
            images.append({"file": image.name, "sha256": digest(raw), "pixels": pixel_size})
    return images


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("paper", choices=("a4", "letter"))
    ap.add_argument("pdf", type=Path)
    ap.add_argument("--fixture", type=Path, default=ROOT/"print-original.weftforge")
    ap.add_argument("--spec", type=Path, default=ROOT/"CHECK-SPEC.json")
    ap.add_argument("--native", type=Path)
    ap.add_argument("--render-dir", type=Path)
    args = ap.parse_args()
    assert digest(args.spec.read_bytes()) == SPEC_SHA, "frozen acceptance rules differ"
    spec = json.loads(args.spec.read_bytes())
    fixture = args.fixture.read_bytes()
    assert digest(fixture) == spec["fixture_sha256"], "original source bytes differ"
    project = json.loads(fixture)
    assert project["version"] == spec["native_version"] and project["project"]["revision"] == spec["revision"]
    pdf = args.pdf.read_bytes()
    tiles, labels = read_pdf(args.pdf, project["project"], spec, args.paper)
    renders = read_with_poppler(args.pdf, args.paper, spec, labels, args.render_dir)
    result = {
        "paper": args.paper, "paper_mm": spec["paper_mm"][args.paper], "pages": spec["pages"],
        "fixture_sha256": spec["fixture_sha256"], "fixture_revision": spec["revision"],
        "pdf_sha256": digest(pdf), "pdf_bytes": len(pdf), "spec_sha256": SPEC_SHA,
        "pypdf_version": pypdf.__version__, "PrintScaling": "None", "calibration_mm": [100, 100],
        "tile_overlap_mm": 10, "pattern_scale": "1:1", "tiles": tiles, "renders": renders,
        "scope": spec["proof_scope"],
    }
    if args.native is not None:
        native = args.native.read_bytes()
        assert native == pdf, ("CLI/native bytes differ", digest(pdf), digest(native))
        result["native_sha256"] = digest(native)
        result["native_byte_equal"] = True
    assert args.fixture.read_bytes() == fixture and args.pdf.read_bytes() == pdf
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
