"""Check that an embedded-buffer glTF matches the independently checked GLB."""
from base64 import b64decode, b64encode
from hashlib import sha256
import json
from pathlib import Path
import struct
import sys

PREFIX = 'data:application/octet-stream;base64,'

def binary_parts(raw):
    assert struct.unpack_from('<4sII', raw) == (b'glTF', 2, len(raw))
    size, kind = struct.unpack_from('<I4s', raw, 12)
    assert size % 4 == 0 and kind == b'JSON'
    document = json.loads(raw[20:20 + size])
    size_bin, kind_bin = struct.unpack_from('<I4s', raw, 20 + size)
    assert kind_bin == b'BIN\0' and size_bin % 4 == 0
    buffer = raw[28 + size:]
    assert len(buffer) == size_bin
    assert len(document['buffers']) == 1
    actual_len = document['buffers'][0]['byteLength']
    assert 0 <= len(buffer) - actual_len <= 3 and not any(buffer[actual_len:])
    return document, buffer[:actual_len]

def compare(json_file, glb_file):
    json_raw = json_file.read_bytes()
    glb_raw = glb_file.read_bytes()
    text = json.loads(json_raw)
    original, buffer = binary_parts(glb_raw)
    assert len(text['buffers']) == 1
    uri = text['buffers'][0].pop('uri')
    assert uri.startswith(PREFIX)
    encoded = uri.removeprefix(PREFIX)
    actual = b64decode(encoded, validate=True)
    assert b64encode(actual).decode() == encoded
    assert len(actual) == text['buffers'][0]['byteLength']
    assert actual == buffer, 'binary attribute bytes differ'
    assert text == original, 'scene JSON differs outside the embedded buffer URI'
    return {
        'json_sha256': sha256(json_raw).hexdigest(), 'glb_sha256': sha256(glb_raw).hexdigest(),
        'binary_bytes': len(buffer), 'binary_sha256': sha256(buffer).hexdigest(),
        'same_geometry_and_appearance': True, 'pieces': [node['name'] for node in text['nodes']],
    }

if __name__ == '__main__':
    print(json.dumps(compare(*map(Path, sys.argv[1:])), indent=2))
