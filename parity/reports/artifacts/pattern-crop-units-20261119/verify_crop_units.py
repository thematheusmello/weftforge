import argparse
from hashlib import sha256
import json
from pathlib import Path
import struct
import subprocess
import tempfile
import zlib

from PIL import Image, ImageChops

directory = Path(__file__).resolve().parent
default_fixture = directory.parent / "pattern-print-pdf-20261119" / "print-original.weftforge"
parser = argparse.ArgumentParser(description="Check the scale, crop origin and contours in two pattern PNGs")
parser.add_argument("--fixture", type=Path, default=default_fixture)
parser.add_argument("--cli", type=Path, help="Compare each saved native image with a new CLI export")
args = parser.parse_args()
fixture = args.fixture.resolve()
project_bytes = fixture.read_bytes()
project = json.loads(project_bytes)["project"]
points = {point["id"]: point["position"] for point in project["points"]}
curves = {curve["id"]: curve for curve in project["curves"]}
basin = next(piece for piece in project["pieces"] if piece["name"].startswith("Basin"))
hole, = basin["holes"]
density = 11812


def png_metadata(raw):
    assert raw[:8] == b"\x89PNG\r\n\x1a\n"
    size = struct.unpack_from(">II", raw, 16)
    offset = 8
    phys = None
    while offset < len(raw):
        length, tag = struct.unpack_from(">I4s", raw, offset)
        data = raw[offset + 8:offset + 8 + length]
        crc, = struct.unpack_from(">I", raw, offset + 8 + length)
        assert crc == zlib.crc32(tag + data)
        if tag == b"pHYs":
            assert phys is None
            phys = struct.unpack(">IIB", data)
        offset += 12 + length
    assert offset == len(raw)
    return size, phys


def dark_at(image, point):
    px = round((point[0] + .43) * density)
    py = round((.45 - point[1]) * density)
    assert 2 <= px < image.width - 2 and 2 <= py < image.height - 2
    return image.crop((px - 2, py - 2, px + 3, py + 3)).getextrema()[0] < 240


samples = []
for curve_id in basin["boundary"] + hole["boundary"]:
    curve = curves[curve_id]
    assert curve["controls"] is None
    start, end = points[curve["start"]], points[curve["end"]]
    samples.extend([
        [a + (b - a) * step for a, b in zip(start, end)]
        for step in (0., .25, .5, .75)
    ])

images = {}
outputs = []
for name, width, right in (
    ("physical", 1654, -.29),
    ("pixel-edit", 1680, (-430. + 1680. * (1000. / 11812.)) / 1000.),
):
    target = directory / f"{name}.png"
    options = directory / f"{name}.json"
    request = json.loads(options.read_text())
    assert request == {
        "crop_m": [[-.43, .275], [right, .45]],
        "pixels_per_inch": 300.012101,
        "contours": True,
        "internal_lines": True,
        "notches": True,
    }
    raw = target.read_bytes()
    if args.cli:
        with tempfile.TemporaryDirectory() as temp:
            fresh = Path(temp) / "export.png"
            subprocess.run(
                [str(args.cli.resolve()), "export-pattern-png", str(fixture), str(fresh), str(options)],
                check=True,
            )
            assert fresh.read_bytes() == raw, name
    size = (width, 2067)
    assert png_metadata(raw) == (size, (density, density, 1))
    with Image.open(target) as source:
        assert source.mode == "RGBA" and source.size == size and source.getextrema()[3] == (255, 255)
        image = source.convert("L")
    assert all(dark_at(image, point) for point in samples)
    assert not dark_at(image, (-.37, .337))
    assert not dark_at(image, (-.425, .445))
    images[name] = image
    outputs.append({
        "mode": name, "size_px": size, "ppm": density, "crop_m": request["crop_m"],
        "outline_samples": len(samples), "sha256": sha256(raw).hexdigest(), "bytes": len(raw),
    })
physical, edited = images["physical"], images["pixel-edit"]
assert ImageChops.difference(physical, edited.crop((0, 0, 1654, 2067))).getbbox() is None
assert edited.crop((1654, 0, 1680, 2067)).getextrema() == (255, 255)
assert fixture.read_bytes() == project_bytes
print(json.dumps({
    "project_sha256": sha256(project_bytes).hexdigest(),
    "cli_sha256": sha256(args.cli.read_bytes()).hexdigest() if args.cli else None,
    "native_byte_identical": args.cli is not None,
    "shared_crop_pixels_exact": True,
    "new_white_columns": 26,
    "outputs": outputs,
}, indent=2))
