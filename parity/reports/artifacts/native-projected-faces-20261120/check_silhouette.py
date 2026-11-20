"""Check two native Harbor workspaces against the independently projected cloth.

Usage: python3 check_silhouette.py original.obj corrected.obj original.png corrected.png
Requires Pillow. The native capture is 1640 by 1000, with the default orbit.
"""
import argparse
import hashlib
import json
import math
import operator
from pathlib import Path

from PIL import Image, ImageChops, ImageDraw, ImageFilter


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def garment_mask(obj):
    # The 1640 × 1000 native workspace allocates this canvas at [698, 143, 1348, 944].
    center = (1023.0, 543.5)
    scale = 270.0
    yaw, pitch = -0.45, 0.12
    positions, points, faces = [], [], []
    for line in obj.read_text().splitlines():
        words = line.split()
        if not words:
            continue
        if words[0] == 'v':
            x, y, z = map(float, words[1:])
            positions.append((x, y, z))
            right = math.cos(yaw) * x + math.sin(yaw) * z
            depth = -math.sin(yaw) * x + math.cos(yaw) * z
            down = -math.cos(pitch) * (y - 1.0) + math.sin(pitch) * depth
            points.append((center[0] + scale * right, center[1] + scale * down))
        elif words[0] == 'f':
            faces.append([int(corner.split('/')[0]) - 1 for corner in words[1:]])
    assert len(points) == 644 and len(faces) == 1150
    mask = Image.new('L', (1640, 1000))
    draw = ImageDraw.Draw(mask)
    for face in faces:
        assert len(face) == 3
        draw.polygon([points[i] for i in face], fill=255)
    bounds = [min(x for x, _ in points), min(y for _, y in points),
              max(x for x, _ in points), max(y for _, y in points)]
    return mask, bounds, positions, faces


def original_face_colors(image, positions, faces):
    # Native pixels well away from the cloth, sampled on the two visible rays.
    samples = [(192, (957, 225)), (1067, (926, 180))]
    observations = []
    with Image.open(image) as source:
        rgb = source.convert('RGB')
    for face_number, pixel in samples:
        a, b, c = [positions[index] for index in faces[face_number - 1]]
        e = tuple(map(operator.sub, b, a))
        h = tuple(map(operator.sub, c, a))
        normal = (e[1] * h[2] - e[2] * h[1],
                  e[2] * h[0] - e[0] * h[2],
                  e[0] * h[1] - e[1] * h[0])
        dot = sum(x * y for x, y in zip(normal, (0.3, 0.5, 0.8)))
        light = 0.45 + 0.55 * min(1.0, abs(dot / math.hypot(*normal)))
        expected = tuple(int(255 * light * component) for component in (0.22, 0.55, 0.76))
        actual = rgb.getpixel(pixel)
        assert expected == actual
        observations.append(dict(obj_face=face_number, pixel=pixel, material_rgb=actual))
    return observations


def measure(image, mask):
    with Image.open(image) as source:
        rgb = source.convert('RGB')
    assert rgb.size == mask.size
    blue = Image.new('L', rgb.size)
    # The mannequin's maximum blue–red difference is 21. Neutral grid and HUD
    # are also below this threshold. Exclude the toolbar and inspector.
    source, dest = rgb.load(), blue.load()
    for y in range(175, 944):
        for x in range(698, 1348):
            r, g, b = source[x, y]
            if b - r > 30 and g - r > 14:
                dest[x, y] = 255
    # A two-pixel margin accommodates coverage at triangle boundaries.
    outside = ImageChops.subtract(blue, mask.filter(ImageFilter.MaxFilter(5)))
    return {'image_sha256': digest(image), 'material_pixels': blue.histogram()[255],
            'material_pixels_outside_silhouette_margin': outside.histogram()[255],
            'outside_pixel_bounds_exclusive': outside.getbbox()}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    for arg in ('original_obj', 'corrected_obj', 'original_png', 'corrected_png'):
        parser.add_argument(arg, type=Path)
    args = parser.parse_args()
    assert digest(args.original_png) == '3605fb9ce7fea89fe6cdccdf8b0c174334c19d2b64afd96672504fc55dcf39c4'
    assert digest(args.original_obj) == '8cec02e8988d1a37d5797e63f6de77df509ca5a7a6604fb4c8ba91aefc8eb466'
    assert args.original_obj.read_bytes() == args.corrected_obj.read_bytes()
    mask, bounds, positions, faces = garment_mask(args.corrected_obj)
    samples = original_face_colors(args.original_png, positions, faces)
    old, new = measure(args.original_png, mask), measure(args.corrected_png, mask)
    print(json.dumps({'obj_sha256': digest(args.corrected_obj), 'vertices': 644, 'faces': 1150,
                      'projected_bounds': [round(x, 3) for x in bounds],
                      'original_face_colors': samples, 'original': old, 'corrected': new}, indent=2))
    assert old['material_pixels_outside_silhouette_margin'] > 500
    assert new['material_pixels'] > 10000
    assert new['material_pixels_outside_silhouette_margin'] == 0
