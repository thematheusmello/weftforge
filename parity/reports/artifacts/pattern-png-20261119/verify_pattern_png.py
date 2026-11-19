import argparse
from hashlib import sha256
from pathlib import Path
import json
import math
import struct
import subprocess
import tempfile
import zlib

from PIL import Image

out = Path(__file__).resolve().parent
parser = argparse.ArgumentParser(description='Check physical crop and authored marks in the recorded pattern PNGs')
parser.add_argument('--cli', type=Path, help='Compare a fresh export with the saved native PNGs')
cli = parser.parse_args().cli
fixture = out.parent/'pattern-print-pdf-20261119'/'print-original.weftforge'
project_bytes = fixture.read_bytes()
project = json.loads(project_bytes)['project']
points = {r['id']: r['position'] for r in project['points']}
curves = {r['id']: r for r in project['curves']}
big, small = project['pieces']
crop = [[-.422, -.281], [.447, .442]]
density = 5000
size = (4345, 3615)

def sample(curve_id, t):
    curve = curves[curve_id]
    a,b = points[curve['start']], points[curve['end']]
    if curve['controls']:
        c,d = curve['controls']
        return [u*(1-t)**3 + 3*v*(1-t)**2*t + 3*w*(1-t)*t*t + x*t**3 for u,v,w,x in zip(a,c,d,b)]
    return [u+(v-u)*t for u,v in zip(a,b)]

def ink(image, point):
    x,y = point
    px,py = round((x-crop[0][0])*density), round((crop[1][1]-y)*density)
    assert 2 <= px < size[0]-2 and 2 <= py < size[1]-2
    return image.crop((px-2,py-2,px+3,py+3)).getextrema()[0] < 240

def phys(raw):
    assert raw[:8] == b'\x89PNG\r\n\x1a\n'
    assert struct.unpack_from('>II', raw, 16) == size
    chunks = {}
    offset = 8
    while offset < len(raw):
        n,tag = struct.unpack_from('>I4s', raw, offset)
        data = raw[offset+8:offset+8+n]
        crc, = struct.unpack_from('>I', raw, offset+8+n)
        assert crc == zlib.crc32(tag + data)
        if tag == b'pHYs':
            assert tag not in chunks
            chunks[tag] = struct.unpack('>IIB',data)
        offset += 12+n
    assert offset == len(raw) and chunks[b'pHYs'] == (density,density,1)

notch, = big['notches']
mark = sample(notch['curve'], notch['t'])
curve = curves[notch['curve']]
a,b = points[curve['start']],points[curve['end']]
dx,dy = (v-u for u,v in zip(a,b))
mag = math.hypot(dx,dy)
notch_probe = (mark[0] - .003*dy/mag, mark[1] + .003*dx/mag)
contours = [*big['boundary'], *small['boundary'], *small['holes'][0]['boundary']]
curve_samples = [sample(id,t) for id in contours for t in (.2,.4,.55,.8)]
details = [notch_probe, sample(big['internal_curves'][0],.49)]
rows = []
for mode, shown in [('all-marks',True),('contours-only',False)]:
    options = out/f'{mode}.json'
    target = out/f'{mode}.png'
    request = json.loads(options.read_text())
    assert request == dict(crop_m=crop,pixels_per_inch=127,contours=True,internal_lines=shown,notches=shown)
    raw = target.read_bytes()
    if cli is not None:
        with tempfile.TemporaryDirectory() as temp:
            destination = Path(temp)/'fresh.png'
            subprocess.run([str(cli.resolve()),'export-pattern-png',str(fixture),str(destination),str(options)],check=True)
            assert destination.read_bytes() == raw, mode
    phys(raw)
    with Image.open(target) as source:
        assert source.mode == 'RGBA' and source.size == size and source.getextrema()[3] == (255,255)
        mono = source.convert('L')
    assert all(ink(mono,p) for p in curve_samples)
    assert [ink(mono,p) for p in details] == [shown,shown]
    assert not ink(mono,(-.36,.35))
    rows.append(dict(mode=mode,sha256=sha256(raw).hexdigest(),bytes=len(raw),size_px=size,
                     ppm=density,outline_samples=len(curve_samples),details_shown=shown))
assert fixture.read_bytes() == project_bytes
print(json.dumps(dict(project_sha256=sha256(project_bytes).hexdigest(),outputs=rows,
                      fresh_cli_compare=cli is not None),indent=2))
