import argparse
from hashlib import sha256
from pathlib import Path
import json
import math
import subprocess
import tempfile

from PIL import Image, ImageChops

folder = Path(__file__).resolve().parent
parser = argparse.ArgumentParser(description="Check the physical location and direction of the rotated warp arrow")
parser.add_argument("--fixture", type=Path, default=folder / "rotated-basin.weftforge")
parser.add_argument("--legacy", type=Path, default=folder.parent / "pattern-crop-units-20261119" / "physical.png")
parser.add_argument("--cli", type=Path, help="Compare a fresh export with each native file")
args = parser.parse_args()
project_bytes = args.fixture.read_bytes()
project = json.loads(project_bytes)["project"]
basin = next(piece for piece in project["pieces"] if piece["id"] == 11)
assert project["revision"] == 3 and basin["grain_angle"] == math.pi / 2
vertices = {point["id"]: point["position"] for point in project["points"]}
curves = {curve["id"]: curve for curve in project["curves"]}
boundary = [vertices[curves[c]["start"]] for c in basin["boundary"]]
assert all(curves[c]["controls"] is None for c in basin["boundary"])
x0, y0 = map(min, zip(*boundary))
x1, y1 = map(max, zip(*boundary))
center_x, center_y = (x0 + x1) / 2, (y0 + y1) / 2
shaft = (x1 - x0) / 4
assert x1 - x0 < y1 - y0 and shaft < .1
tip_y = center_y + shaft
right_wing = math.sin(math.pi / 5) * shaft / 4
down_wing = math.cos(math.pi / 5) * shaft / 4
ppm = 11812

def screen(x, y):
    return ((x + .43) * ppm, (.45 - y) * ppm)

def dark_at(image, x, y):
    px, py = map(round, screen(x, y))
    return image.crop((px - 2, py - 2, px + 3, py + 3)).getextrema()[0] < 240

images, outputs = {}, []
for name in ("without-grainline", "with-grainline"):
    enabled = name == "with-grainline"
    image_path = folder / f"{name}.png"
    options_path = folder / f"{name}.json"
    options = {
        "crop_m": [[-.43, .275], [-.29, .45]], "pixels_per_inch": 300.012101,
        "contours": True, "internal_lines": True, "notches": True,
    }
    if enabled:
        options["grainline"] = True
    assert json.loads(options_path.read_text()) == options
    raw = image_path.read_bytes()
    if not enabled:
        assert raw == args.legacy.read_bytes()
    if args.cli:
        with tempfile.TemporaryDirectory() as temp:
            fresh = Path(temp) / "export.png"
            subprocess.run([str(args.cli.resolve()), "export-pattern-png", str(args.fixture.resolve()),
                            str(fresh), str(options_path)], check=True)
            assert fresh.read_bytes() == raw
    with Image.open(image_path) as stream:
        assert (stream.format, stream.mode, stream.size) == ("PNG", "RGBA", (1654, 2067))
        assert all(round(dpi / .0254) == ppm for dpi in stream.info["dpi"])
        assert stream.getextrema()[3] == (255, 255)
        images[enabled] = stream.convert("L")
    outputs.append({"mode": name, "sha256": sha256(raw).hexdigest(), "bytes": len(raw)})

without, with_grain = images[False], images[True]
probes = [
    (center_x, center_y + shaft * .25),
    (center_x, center_y + shaft * .75),
    (center_x - right_wing * .75, tip_y - down_wing * .75),
    (center_x + right_wing * .75, tip_y - down_wing * .75),
]
assert all(dark_at(with_grain, x, y) and not dark_at(without, x, y) for x, y in probes)
assert not dark_at(with_grain, center_x + shaft * .5, center_y)
assert not dark_at(with_grain, center_x, center_y - shaft * .5)
changed = ImageChops.difference(without, with_grain)
left, top, right, bottom = changed.getbbox()
x_min, y_min = screen(center_x - right_wing, tip_y)
x_max, y_max = screen(center_x + right_wing, center_y)
assert x_min - 2 <= left < right <= x_max + 2
assert y_min - 2 <= top < bottom <= y_max + 2
changed_pixels = sum(changed.histogram()[1:])
assert changed_pixels > 300 and args.fixture.read_bytes() == project_bytes
print(json.dumps({
    "project_sha256": sha256(project_bytes).hexdigest(),
    "cli_sha256": sha256(args.cli.read_bytes()).hexdigest() if args.cli else None,
    "native_byte_identical": args.cli is not None, "omitted_matches_prior_export": True,
    "size_px": [1654, 2067], "pixels_per_metre": ppm, "warped_piece_id": 11,
    "authored_warp_angle_rad": basin["grain_angle"], "independent_arrow_probes": len(probes),
    "changed_pixels": changed_pixels, "changed_rect_px": [left, top, right, bottom],
    "outputs": outputs,
}, indent=2))
