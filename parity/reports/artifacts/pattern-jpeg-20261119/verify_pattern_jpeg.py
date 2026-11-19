import argparse
from decimal import Decimal, ROUND_HALF_UP
from hashlib import sha256
import json
import math
from pathlib import Path
import struct
import subprocess
import tempfile

from PIL import Image

folder = Path(__file__).resolve().parent
grainline_archive = folder.parent / "pattern-png-grainline-20261119"
parser = argparse.ArgumentParser(description="Check JPEG density, dimensions and authored pattern marks")
parser.add_argument("--fixture", type=Path, default=grainline_archive / "rotated-basin.weftforge")
parser.add_argument("--png-archive", type=Path, default=grainline_archive)
parser.add_argument("--cli", type=Path, help="Compare fresh CLI and MCP exports and prior PNGs")
args = parser.parse_args()
project_bytes = args.fixture.read_bytes()
project = json.loads(project_bytes)["project"]
assert project["revision"] == 3
points = {point["id"]: point["position"] for point in project["points"]}
curves = {curve["id"]: curve for curve in project["curves"]}
juniper, basin = project["pieces"]
assert basin["id"] == 11 and basin["grain_angle"] == math.pi / 2


def jpeg_header(raw):
    assert raw.startswith(b"\xff\xd8") and raw.endswith(b"\xff\xd9")
    offset = 2
    density = None
    size = None
    while True:
        assert offset + 4 < len(raw) - 2 and raw[offset] == 0xff
        marker, length = struct.unpack_from(">BH", raw, offset + 1)
        end = offset + 2 + length
        assert length >= 2 and end <= len(raw) - 2
        body = raw[offset + 4:end]
        if marker == 0xe0:
            assert density is None and body[:8] == b"JFIF\0\x01\x02\x01"
            assert len(body) == 14 and body[12:] == b"\x00\x00"
            density = struct.unpack_from(">HH", body, 8)
        elif marker == 0xc0:
            assert size is None and len(body) == 9 and (body[0], body[5]) == (8, 1)
            height, width = struct.unpack_from(">HH", body, 1)
            size = (width, height)
        elif marker == 0xda:
            assert end < len(raw) - 2 and body[0] == 1
            assert all(part.startswith(b"\x00") for part in raw[end:-2].split(b"\xff")[1:])
            assert density and size
            return density, size
        else:
            assert marker in (0xdb, 0xc4), f"Unexpected JPEG header marker {marker:02x}"
        offset = end


def sample(curve_id, t):
    curve = curves[curve_id]
    start, end = points[curve["start"]], points[curve["end"]]
    if curve["controls"] is None:
        return [a + (b - a) * t for a, b in zip(start, end)]
    c, d = curve["controls"]
    return [
        a * (1-t)**3 + 3*b * (1-t)**2*t + 3*c * (1-t)*t**2 + d*t**3
        for a, b, c, d in zip(start, c, d, end)
    ]


def patch(image, crop, ppi, point):
    x = round((point[0] - crop[0][0]) * ppi / .0254)
    y = round((crop[1][1] - point[1]) * ppi / .0254)
    assert 2 <= x < image.width - 2 and 2 <= y < image.height - 2, (point, x, y)
    return image.crop((x - 2, y - 2, x + 3, y + 3)).getextrema()[0]


def coordinate_extent(crop, axis, ppi):
    length = Decimal(str(crop[1][axis])) - Decimal(str(crop[0][axis]))
    pixels = length * Decimal(ppi) / Decimal(".0254")
    rounded = pixels.quantize(Decimal(1), rounding=ROUND_HALF_UP)
    return int(rounded), float(abs(pixels - rounded))


notch, = juniper["notches"]
line = curves[notch["curve"]]
assert line["controls"] is None
marker_origin = sample(notch["curve"], notch["t"])
dx, dy = [b-a for a,b in zip(points[line["start"]], points[line["end"]])]
notch_probe = (
    marker_origin[0] - .003 * dy / math.hypot(dx, dy),
    marker_origin[1] + .003 * dx / math.hypot(dx, dy),
)
annotation_probe = sample(juniper["internal_curves"][0], .49)

# The author's Basin outline is a rectangle. Its upward shaft and wings use the recorded warp angle.
vertices = [points[curves[curve]["start"]] for curve in basin["boundary"]]
xs, ys = zip(*vertices)
cx, cy = (max(xs) + min(xs)) / 2, (max(ys) + min(ys)) / 2
shaft = (max(xs) - min(xs)) / 4
wing, back = math.sin(math.pi / 5) * shaft / 4, math.cos(math.pi / 5) * shaft / 4
grain_probes = [
    (cx, cy + .25 * shaft), (cx, cy + .75 * shaft),
    (cx - .75 * wing, cy + shaft - .75 * back),
    (cx + .75 * wing, cy + shaft - .75 * back),
]

files = (("fit", "fit.jpg", 127), ("two-metre", "two-metre.jpeg", 72))
requests, outputs = {}, []
for name, filename, ppi in files:
    target = folder / filename
    options = json.loads((folder / f"{name}.json").read_text())
    requests[name] = options
    expected_crop = [[-.422, -.281], [.447, .442]] if name == "fit" else [[-1., .275], [1., .475]]
    assert options == {
        "crop_m": expected_crop, "pixels_per_inch": 127 if name == "fit" else 72.4,
        "contours": True, "internal_lines": True, "notches": True, "grainline": True,
    }
    size, error = zip(*(coordinate_extent(expected_crop, axis, ppi) for axis in (0, 1)))
    assert all(value <= .5 for value in error)
    raw = target.read_bytes()
    assert jpeg_header(raw) == ((ppi, ppi), size)
    curves_in_crop = basin["boundary"] + basin["holes"][0]["boundary"]
    if name == "fit":
        curves_in_crop = juniper["boundary"] + curves_in_crop
    outline_samples = [
        sample(curve_id, step / 16)
        for curve_id in curves_in_crop for step in range(1, 16)
    ]
    details = grain_probes + ([notch_probe, annotation_probe] if name == "fit" else [])
    with Image.open(target) as image:
        assert (image.format, image.mode, image.size, image.info["dpi"]) == ("JPEG", "L", size, (ppi, ppi))
        assert all(patch(image, expected_crop, ppi, point) < 240 for point in outline_samples + details)
        assert all(patch(image, expected_crop, ppi, point) >= 250 for point in [(-.37, .337), (-.425, .445)] if point[1] < expected_crop[1][1])
    outputs.append({
        "name": name, "file": filename, "size_px": size, "jfif_ppi": ppi,
        "physical_side_error_px": error, "outline_samples": len(outline_samples),
        "mark_samples": len(details), "sha256": sha256(raw).hexdigest(), "bytes": len(raw),
    })

cli_sha = None
if args.cli:
    cli = args.cli.resolve()
    fixture = args.fixture.resolve()
    cli_sha = sha256(cli.read_bytes()).hexdigest()

    def export(command, destination, options_path):
        subprocess.run([str(cli), command, str(fixture), str(destination), str(options_path)], check=True)

    with tempfile.TemporaryDirectory() as temp:
        temp = Path(temp)
        mcp_requests = [
            {"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
                "protocolVersion": "2025-11-25", "capabilities": {},
                "clientInfo": {"name": "independent-jpeg-check", "version": "1"},
            }},
            {"jsonrpc": "2.0", "method": "notifications/initialized"},
        ]
        for identifier, (name, filename, _) in enumerate(files, 2):
            exporter = temp / ("cli-" + filename)
            export("export-pattern-jpeg", exporter, folder / f"{name}.json")
            assert exporter.read_bytes() == (folder / filename).read_bytes()
            mcp_requests.append({"jsonrpc": "2.0", "id": identifier, "method": "tools/call", "params": {
                "name": "project_export_pattern_jpeg",
                "arguments": {"input": str(fixture), "output": str(temp / ("mcp-" + filename)),
                              "options": requests[name]},
            }})
        invalid = requests["two-metre"] | {"pixels_per_inch": .4}
        mcp_requests.append({"jsonrpc": "2.0", "id": 4, "method": "tools/call", "params": {
            "name": "project_export_pattern_jpeg",
            "arguments": {"input": str(fixture), "output": str(temp / "mcp-two-metre.jpeg"),
                          "options": invalid},
        }})
        response = subprocess.run(
            [str(cli), "mcp"], input="\n".join(map(json.dumps, mcp_requests)) + "\n",
            stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, timeout=90, check=True,
        )
        messages = [json.loads(line) for line in response.stdout.splitlines()]
        assert not response.stderr and [item["id"] for item in messages] == [1, 2, 3, 4]
        assert messages[0]["result"]["protocolVersion"] == "2025-11-25"
        assert messages[-1]["result"]["isError"]
        for observed, message in zip(outputs, messages[1:3], strict=True):
            result = message["result"]["structuredContent"]
            assert result["kind"] == "pattern_jpeg" and result["source_revision"] == 3
            assert result["size_px"] == list(observed["size_px"])
            assert result["effective_pixels_per_inch"] == observed["jfif_ppi"]
            assert result["options"] == requests[observed["name"]]
            filename = observed["file"]
            assert (temp / ("mcp-" + filename)).read_bytes() == (folder / filename).read_bytes()
        for name in ("without-grainline", "with-grainline"):
            fresh = temp / f"{name}.png"
            export("export-pattern-png", fresh, args.png_archive / f"{name}.json")
            assert fresh.read_bytes() == (args.png_archive / fresh.name).read_bytes()
        png_options = temp / "at-72.json"
        png_options.write_text(json.dumps(requests["two-metre"] | {"pixels_per_inch": 72}))
        wide_png = temp / "two-metre.png"
        export("export-pattern-png", wide_png, png_options)
        with Image.open(wide_png) as source:
            assert source.size == (5670, 567)
assert args.fixture.read_bytes() == project_bytes
print(json.dumps({
    "project_sha256": sha256(project_bytes).hexdigest(), "cli_sha256": cli_sha,
    "native_cli_mcp_byte_identical": cli_sha is not None,
    "prior_pngs_byte_identical": cli_sha is not None,
    "two_metre_control_png_width_px": 5670 if cli_sha else None,
    "invalid_mcp_preserved_output": cli_sha is not None, "outputs": outputs,
}, indent=2))
