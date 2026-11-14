#!/usr/bin/env python3
"""FILE-005-v1 independent oracle; never imports or invokes WeftForge."""
import argparse
from collections import Counter
from copy import deepcopy
from hashlib import sha256
import importlib.metadata
import json
import math
from pathlib import Path
import re
import struct
import sys

import ezdxf
from ezdxf.math import BSpline

ROOT = Path(__file__).resolve().parent
SAMPLES = (0, 0.125, 0.25, 0.5, 0.75, 0.875, 1)
TOLERANCE_MM = 1e-8
SECTIONS = ['HEADER', 'CLASSES', 'TABLES', 'BLOCKS', 'ENTITIES', 'OBJECTS']
TABLES = ['VPORT', 'LTYPE', 'LAYER', 'STYLE', 'VIEW', 'UCS', 'APPID', 'DIMSTYLE', 'BLOCK_RECORD']
SUBCLASSES = {
    'VPORT': 'AcDbViewportTableRecord', 'LTYPE': 'AcDbLinetypeTableRecord',
    'LAYER': 'AcDbLayerTableRecord', 'STYLE': 'AcDbTextStyleTableRecord',
    'VIEW': 'AcDbViewTableRecord', 'UCS': 'AcDbUCSTableRecord',
    'APPID': 'AcDbRegAppTableRecord', 'DIMSTYLE': 'AcDbDimStyleTableRecord',
    'BLOCK_RECORD': 'AcDbBlockTableRecord',
}
KNOTS = [0.0] * 4 + [1.0] * 4


def digest(data):
    return sha256(data).hexdigest()


def values(tags, code):
    return [v for c, v in tags if c == code]


def one(tags, code):
    found = values(tags, code)
    assert len(found) == 1, f'group {code}: expected once, found {found!r}'
    return found[0]


def number(value):
    result = float(value)
    assert math.isfinite(result), f'non-finite number {value!r}'
    return result


def handle(value):
    assert re.fullmatch(r'[0-9a-fA-F]+', value), f'invalid hexadecimal handle {value!r}'
    return int(value, 16)


def subtype(tags, name):
    start = tags.index((100, name)) + 1
    end = next((i for i in range(start, len(tags)) if tags[i][0] == 100), len(tags))
    return tags[start:end]


def owner(tags):
    outside = []
    nesting = 0
    for code, value in tags:
        if code == 102:
            nesting += 1 if value.startswith('{') else -1
            assert nesting >= 0, 'unbalanced 102 group'
        elif nesting == 0:
            outside.append((code, value))
    assert nesting == 0, 'unbalanced 102 group'
    return handle(values(outside, 330)[0])


def records(tags):
    result = []
    for tag in tags:
        if tag[0] == 0:
            result.append([tag])
        else:
            assert result, f'tag outside record {tag!r}'
            result[-1].append(tag)
    return result


def parse_raw(data):
    assert data.endswith(b'\n'), 'missing final LF'
    assert b'\r' not in data, 'non-LF line ending'
    assert b'\x00' not in data and not data.startswith(b'\xef\xbb\xbf'), 'binary/BOM'
    lines = data.decode('ascii').splitlines()
    assert len(lines) % 2 == 0, 'odd group/value line count'
    tags = []
    for i in range(0, len(lines), 2):
        assert re.fullmatch(r'\s*\d+\s*', lines[i]), f'invalid group code line {i + 1}'
        code = int(lines[i])
        assert 0 <= code <= 1071, f'invalid group code {code}'
        tags.append((code, lines[i + 1].strip()))
    comments = values(tags, 999)
    tags = [t for t in tags if t[0] != 999]
    sections = {}
    position = 0
    for name in SECTIONS:
        assert tags[position:position + 2] == [(0, 'SECTION'), (2, name)], f'missing/out-of-order section {name}'
        end = tags.index((0, 'ENDSEC'), position + 2)
        content = tags[position + 2:end]
        assert (0, 'SECTION') not in content, f'nested section {name}'
        sections[name] = content
        position = end + 1
    assert tags[position:] == [(0, 'EOF')], 'extra/missing sections, EOF or trailing tags'
    return sections, comments


def authored(source):
    project = source['project']
    assert source['format'] == 'weftforge' and source['version'] == 2
    points = {p['id']: p['position'] for p in project['points']}
    curves = {c['id']: c for c in project['curves']}
    result = []
    for piece in sorted(project['pieces'], key=lambda p: p['id']):
        assert piece['id'] > 0
        chain = []
        for cid in piece['boundary']:
            c = curves[cid]
            p0, p3 = points[c['start']], points[c['end']]
            cp = c['controls']
            assert cp is None or len(cp) == 2
            points2 = [p0, p3] if cp is None else [p0, *cp, p3]
            points3 = [tuple(number(v) * 1000.0 for v in xy) + (0.0,) for xy in points2]
            assert all(math.isfinite(v) for xyz in points3 for v in xyz)
            chain.append({'kind': 'LINE' if cp is None else 'SPLINE', 'piece': piece['id'],
                          'curve': cid, 'layer': f"WF_PIECE_{piece['id']}", 'points': points3})
        assert chain
        assert all(a['points'][-1] == b['points'][0] for a, b in zip(chain, chain[1:] + chain[:1]))
        result.extend(chain)
    return result


def raw_point(tags, code, index=0):
    return tuple(number(values(tags, code + offset)[index]) for offset in (0, 10, 20))


def bezier(cp, t):
    return tuple((1-t)**3 * cp[0][axis] + 3*(1-t)**2*t*cp[1][axis]
                 + 3*(1-t)*t*t*cp[2][axis] + t**3*cp[3][axis] for axis in range(3))


def check_geometry(tags, expected):
    assert one(tags, 0) == expected['kind'], f"curve {expected['curve']}: entity type"
    assert one(tags, 8) == expected['layer'], f"curve {expected['curve']}: layer/order"
    assert values(tags, 100) == ['AcDbEntity', 'AcDbLine' if expected['kind'] == 'LINE' else 'AcDbSpline'], 'entity subclasses'
    assert int(one(tags, 67)) == 0 and one(tags, 410) == 'Model', 'model-space declarations'
    assert int(one(tags, 370)) == -1, 'lineweight must inherit layer'
    assert not values(tags, 62) or int(one(tags, 62)) == 256, 'color must inherit layer'
    assert not values(tags, 6) or one(tags, 6).upper() == 'BYLAYER', 'linetype must inherit layer'
    assert not values(tags, 39) or number(one(tags, 39)) == 0, 'nonzero thickness'
    if expected['kind'] == 'LINE':
        for code in (10, 20, 30, 11, 21, 31):
            one(tags, code)
        actual = [raw_point(tags, 10), raw_point(tags, 11)]
        assert actual == expected['points'], f"curve {expected['curve']}: exact scaled LINE endpoints {actual!r} != {expected['points']!r}"
    else:
        assert [int(one(tags, c)) for c in (70, 71, 72, 73, 74)] == [8, 3, 8, 4, 0], 'SPLINE flags/degree/counts'
        assert [number(v) for v in values(tags, 40)] == KNOTS, 'SPLINE knots'
        assert all(len(values(tags, c)) == 4 for c in (10, 20, 30)), 'SPLINE control point tag counts'
        actual = [raw_point(tags, 10, i) for i in range(4)]
        assert actual == expected['points'], f"curve {expected['curve']}: exact scaled SPLINE controls {actual!r} != {expected['points']!r}"
        assert tuple(number(one(tags, c)) for c in (210, 220, 230)) == (0, 0, 1), 'SPLINE normal'
        assert not any(values(tags, c) for c in (41, 11, 21, 31, 12, 22, 32, 13, 23, 33)), 'weights/fit points/tangents must be omitted'
        for code, default in ((42, 1e-7), (43, 1e-7), (44, 1e-10)):
            assert not values(tags, code) or number(one(tags, code)) == default, 'SPLINE nondefault tolerance'
    return actual


def inspect_raw(data, source):
    errors, details = [], {}

    def run(label, fn):
        try:
            return fn()
        except Exception as exc:
            errors.append(f'{label}: {type(exc).__name__}: {exc}')
            return None

    parsed = run('ASCII/sections', lambda: parse_raw(data))
    if parsed is None:
        return errors, details
    sections, comments = parsed
    expected = authored(source)
    wanted_layers = ['0'] + [f"WF_PIECE_{p['id']}" for p in sorted(source['project']['pieces'], key=lambda p: p['id'])]
    details['sections'] = list(sections)
    details['comments'] = comments

    def check_comments():
        text = '\n'.join(comments)
        lowered = text.lower()
        assert 'WeftForge' in text and 'FILE-005-v1' in text, 'missing producer/profile comments'
        assert re.search(r'\brevision\s*[:=]?\s*' + str(source['project']['revision']) + r'\b', text, re.I), 'source revision comment'
        assert any(word in lowered for word in ('millimetre', 'millimeter', ' mm')), 'missing millimetre comment'
        assert re.search(r'u\s*[/,]?\s*v', lowered) and re.search(r'x\s*[/,]?\s*y', lowered), 'missing UV/XY mapping comment'
        assert ('boundary' in lowered or 'boundaries' in lowered) and 'only' in lowered, 'missing boundary-only scope comment'
        assert not re.search(r'\d{4}-\d\d-\d\d|/Users/|/home/|\\Users\\|\.weftforge', text), 'timestamp/path in comments'
        for piece in source['project']['pieces']:
            assert piece['name'] not in text, 'piece name leaked'
    run('comments', check_comments)

    header = {}
    def check_header():
        for code, value in sections['HEADER']:
            if code == 9:
                assert value not in header, f'duplicate header {value}'
                header[value] = []
            else:
                assert header, 'header value without variable'
                header[next(reversed(header))].append((code, value))
        for key, code, value in [('$ACADVER', 1, 'AC1015'), ('$INSUNITS', 70, '4'), ('$MEASUREMENT', 70, '1'), ('$LUNITS', 70, '2')]:
            assert header.get(key) == [(code, value)], f'{key} missing/wrong raw declaration'
        assert handle(one(header['$HANDSEED'], 5)) > 0, 'invalid HANDSEED'
        assert not any(key in header for key in ('$TDCREATE', '$TDUPDATE', '$FINGERPRINTGUID', '$VERSIONGUID')), 'timestamp/random identity header'
    run('header', check_header)
    run('empty CLASSES', lambda: require(not sections['CLASSES'], 'nonempty CLASSES'))

    tables = {}
    all_records = []
    def check_tables():
        rs = records(sections['TABLES'])
        pos = 0
        names = []
        while pos < len(rs):
            table = rs[pos]
            assert one(table, 0) == 'TABLE', 'record outside TABLE'
            name = one(table, 2)
            assert name not in tables, 'duplicate TABLE'
            names.append(name)
            pos += 1
            entries = []
            while pos < len(rs) and one(rs[pos], 0) != 'ENDTAB':
                entries.append(rs[pos])
                pos += 1
            assert pos < len(rs) and rs[pos] == [(0, 'ENDTAB')], 'missing/nonempty ENDTAB'
            tables[name] = (table, entries)
            all_records.extend([table, *entries])
            pos += 1
            assert int(one(table, 70)) == len(entries), f'{name} count mismatch'
            assert 'AcDbSymbolTable' in values(table, 100), f'{name} table subclass'
            assert owner(table) == 0, f'{name} table root owner'
            th = handle(one(table, 5))
            for entry in entries:
                assert one(entry, 0) == name, f'{name} wrong entry type'
                assert owner(entry) == th, f'{name} table-record owner'
                assert values(entry, 100) == ['AcDbSymbolTableRecord', SUBCLASSES[name]], f'{name} record subclasses'
        assert names == TABLES, f'tables/order {names!r}'
        expected_names = {'LTYPE': ['BYBLOCK', 'BYLAYER', 'CONTINUOUS'], 'LAYER': wanted_layers,
                          'STYLE': ['STANDARD'], 'DIMSTYLE': ['STANDARD'], 'APPID': ['ACAD'],
                          'BLOCK_RECORD': ['*Model_Space', '*Paper_Space']}
        for name, wanted in expected_names.items():
            actual = [one(r, 2) for r in tables[name][1]]
            assert actual == wanted, f'{name} names/order {actual!r} != {wanted!r}'
        for layer in tables['LAYER'][1]:
            assert int(one(layer, 70)) == 0 and int(one(layer, 62)) == 7 and one(layer, 6) == 'CONTINUOUS', 'visible/thawed/unlocked layer fields'
            assert not values(layer, 290) or int(one(layer, 290)) == 1, 'nonplotting layer'
        for record in tables['BLOCK_RECORD'][1]:
            assert not any(values(record, code) for code in (70, 280, 281)), 'R2007 BLOCK_RECORD field in R2000'
        for record in tables['DIMSTYLE'][1]:
            assert len(values(record, 105)) == 1 and not values(record, 5), 'DIMSTYLE handle must be group 105'
    run('TABLES', check_tables)

    block_records = {}
    if 'BLOCK_RECORD' in tables:
        block_records = {one(r, 2): r for r in tables['BLOCK_RECORD'][1]}
    def check_blocks():
        blocks = records(sections['BLOCKS'])
        all_records.extend(blocks)
        assert len(blocks) == 4 and [one(r, 0) for r in blocks] == ['BLOCK', 'ENDBLK'] * 2, 'space BLOCK/ENDBLK pairs only'
        for offset, name in enumerate(['*Model_Space', '*Paper_Space']):
            begin, end = blocks[offset * 2:offset * 2 + 2]
            assert one(begin, 2) == name, 'block name/order'
            bh = handle(one(block_records[name], 5))
            assert owner(begin) == bh and owner(end) == bh, 'block-marker owner'
            assert values(begin, 100) == ['AcDbEntity', 'AcDbBlockBegin'], 'BLOCK subclasses'
            assert values(end, 100) == ['AcDbEntity', 'AcDbBlockEnd'], 'ENDBLK subclasses'
            assert int(one(begin, 70)) == 0, 'external/xref block flags'
            assert raw_point(begin, 10) == (0, 0, 0), 'block origin'
            assert one(begin, 8) == '0' and one(end, 8) == '0', 'block marker layer'
            assert not values(begin, 1) or one(begin, 1) == '', 'block external path'
            assert not values(begin, 3) or one(begin, 3) == name, 'secondary block name'
    run('BLOCKS', check_blocks)

    entity_records = records(sections['ENTITIES'])
    all_records.extend(entity_records)
    details['raw_entity_types'] = [one(r, 0) for r in entity_records]
    details['expected_curves'] = expected
    run('entity count', lambda: require(len(entity_records) == len(expected), f'{len(entity_records)} != {len(expected)}'))
    raw_points = []
    for index, (record, wanted) in enumerate(zip(entity_records, expected)):
        def check_entity(record=record, wanted=wanted):
            assert owner(record) == handle(one(block_records['*Model_Space'], 5)), 'wrong model-space owner'
            return check_geometry(record, wanted)
        points = run(f'entity {index}/curve {wanted["curve"]}', check_entity)
        raw_points.append(points)
    def check_adjacency():
        for piece in source['project']['pieces']:
            indices = [i for i, e in enumerate(expected) if e['piece'] == piece['id']]
            chain = [raw_points[i] for i in indices]
            assert all(chain), 'unreadable geometry prevents closure acceptance'
            assert all(a[-1] == b[0] for a, b in zip(chain, chain[1:] + chain[:1])), f'piece {piece["id"]} adjacency/closure'
    run('exact adjacency/closure', check_adjacency)
    if len(expected) == 4 and all(e['kind'] == 'LINE' for e in expected) and [e['points'] for e in expected] == [
        [(0, 0, 0), (125, 0, 0)], [(125, 0, 0), (125, 250, 0)],
        [(125, 250, 0), (0, 250, 0)], [(0, 250, 0), (0, 0, 0)]]:
        def check_dimension():
            assert raw_points[0][1][0] - raw_points[0][0][0] == 125.0
            assert raw_points[1][1][1] - raw_points[1][0][1] == 250.0
            details['known_rectangle_mm'] = [125.0, 250.0]
        run('known rectangle dimension', check_dimension)

    object_records = records(sections['OBJECTS'])
    all_records.extend(object_records)
    def check_objects():
        assert Counter(one(r, 0) for r in object_records) == Counter({'DICTIONARY': 3, 'LAYOUT': 2}), 'only root/group/layout dictionaries and two layouts'
        dictionaries = [r for r in object_records if one(r, 0) == 'DICTIONARY']
        roots = [r for r in dictionaries if owner(r) == 0]
        assert len(roots) == 1, 'root dictionary'
        root = roots[0]
        rh = handle(one(root, 5))
        by_handle = {handle(one(r, 5)): r for r in object_records}
        def entries(record):
            pairs = []
            for i, tag in enumerate(record):
                if tag[0] == 3:
                    assert i + 1 < len(record) and record[i+1][0] in (350, 360), 'dictionary entry missing reference'
                    pairs.append((tag[1], handle(record[i+1][1])))
            assert len(pairs) == len(values(record, 350)) + len(values(record, 360)), 'unnamed dictionary reference'
            assert len(dict(pairs)) == len(pairs), 'duplicate dictionary entry name'
            return dict(pairs)
        root_entries = entries(root)
        assert set(root_entries) == {'ACAD_GROUP', 'ACAD_LAYOUT'}, 'root entries'
        groups = by_handle[root_entries['ACAD_GROUP']]
        layouts = by_handle[root_entries['ACAD_LAYOUT']]
        assert one(groups, 0) == one(layouts, 0) == 'DICTIONARY', 'dictionary entry target types'
        assert owner(groups) == owner(layouts) == rh, 'child dictionary owners'
        assert entries(groups) == {}, 'ACAD_GROUP not empty'
        layout_entries = entries(layouts)
        assert set(layout_entries) == {'Model', 'Layout1'}, 'layout dictionary entries'
        lh = handle(one(layouts, 5))
        for name, block_name, order in [('Model', '*Model_Space', 0), ('Layout1', '*Paper_Space', 1)]:
            layout = by_handle[layout_entries[name]]
            assert one(layout, 0) == 'LAYOUT' and owner(layout) == lh, 'LAYOUT type/owner'
            assert values(layout, 100) == ['AcDbPlotSettings', 'AcDbLayout'], 'LAYOUT subclasses'
            layout_fields = subtype(layout, 'AcDbLayout')
            assert one(layout_fields, 1) == name and int(one(layout_fields, 71)) == order, 'layout name/tab order'
            block = block_records[block_name]
            assert handle(one(layout_fields, 330)) == handle(one(block, 5)), 'LAYOUT to BLOCK_RECORD link'
            assert handle(one(block, 340)) == handle(one(layout, 5)), 'BLOCK_RECORD to LAYOUT link'
            assert not values(layout_fields, 331) or handle(one(layout_fields, 331)) == 0, 'layout viewport must be absent/null'
            plot = subtype(layout, 'AcDbPlotSettings')
            assert not values(plot, 1) or one(plot, 1) == '', 'claimed page setup'
            assert not values(plot, 2) or one(plot, 2) == '', 'claimed printer'
            assert not values(plot, 4) or one(plot, 4) == '', 'claimed paper size'
        for record in dictionaries:
            assert values(record, 100) == ['AcDbDictionary'], 'DICTIONARY subclass'
    run('OBJECTS/layout links', check_objects)

    def check_handles_and_references():
        handles = {}
        for record in all_records:
            kind = one(record, 0)
            code = 105 if kind == 'DIMSTYLE' else 5
            h = handle(one(record, code))
            assert h > 0, f'{kind} null handle'
            assert h not in handles, f'duplicate handle {h:X}'
            handles[h] = kind
            assert not values(record, 1001), 'XDATA forbidden'
            assert not any(1000 <= c <= 1071 for c, v in record), 'XDATA forbidden'
            for c, v in record:
                if (320 <= c <= 369) or (390 <= c <= 399) or c in (480, 481):
                    reference = handle(v)
                    assert reference == 0 or reference > 0
        for record in all_records:
            for c, v in record:
                if (320 <= c <= 369) or (390 <= c <= 399) or c in (480, 481):
                    ref = handle(v)
                    assert ref == 0 or ref in handles, f'unresolved group {c} reference {v}'
        seed = handle(one(header['$HANDSEED'], 5))
        assert seed > max(handles), 'HANDSEED must exceed every emitted handle'
        details['handle_count'] = len(handles)
        details['max_handle'] = f'{max(handles):X}'
        details['handseed'] = f'{seed:X}'
    run('handles/references', check_handles_and_references)
    return errors, details


def require(condition, message):
    assert condition, message


def consumer_geometry(doc, source):
    expected = authored(source)
    actual = list(doc.modelspace())
    assert len(actual) == len(expected), f'consumer model-space count {len(actual)} != {len(expected)}'
    max_error = 0.0
    observations = []
    for entity, wanted in zip(actual, expected):
        assert entity.dxftype() == wanted['kind'] and entity.dxf.layer == wanted['layer'], 'consumer type/layer/order'
        assert entity.dxf.paperspace == 0, 'consumer paperspace flag'
        if entity.dxftype() == 'LINE':
            points = [tuple(entity.dxf.start), tuple(entity.dxf.end)]
            assert points == wanted['points'], 'consumer exact LINE endpoints'
        else:
            assert entity.dxf.flags == 8 and entity.dxf.degree == 3, 'consumer spline flags/degree'
            assert list(entity.knots) == KNOTS, 'consumer knots'
            points = [tuple(p) for p in entity.control_points]
            assert points == wanted['points'], 'consumer exact SPLINE controls'
            assert tuple(entity.dxf.extrusion) == (0, 0, 1), 'consumer normal'
            assert len(entity.weights) == len(entity.fit_points) == 0, 'consumer weights/fit points'
            spline = entity.construction_tool()
            samples = []
            for t in SAMPLES:
                found = tuple(spline.point(t))
                wanted_xyz = bezier(wanted['points'], t)
                error = max(abs(a-b) for a, b in zip(found, wanted_xyz))
                max_error = max(max_error, error)
                assert error <= TOLERANCE_MM, f'B-spline/Bezier sample t={t}: {error} mm'
                samples.append({'t': t, 'bspline_mm': found, 'authored_bezier_mm': wanted_xyz, 'max_abs_error_mm': error})
            observations.append({'curve': wanted['curve'], 'samples': samples})
    for key, val in [('$ACADVER', 'AC1015'), ('$INSUNITS', 4), ('$MEASUREMENT', 1), ('$LUNITS', 2)]:
        assert doc.header[key] == val, f'consumer header {key}'
    expected_layers = ['0'] + [f"WF_PIECE_{p['id']}" for p in sorted(source['project']['pieces'], key=lambda p: p['id'])]
    # ezdxf may synthesize Defpoints; raw declarations are independently mandatory.
    assert all(name in doc.layers for name in expected_layers), 'consumer declared layers'
    assert set(doc.layouts.names()) == {'Model', 'Layout1'}, 'consumer layouts'
    assert len(doc.layouts.get('Layout1')) == 0, 'consumer paper-space geometry'
    return {'max_bspline_sample_error_mm': max_error, 'splines': observations,
            'modelspace_count': len(actual), 'modelspace_types': [e.dxftype() for e in actual]}


def evaluate(source_path, dxf_path):
    source_bytes = source_path.read_bytes()
    source = json.loads(source_bytes)
    original = dxf_path.read_bytes()
    raw_errors, raw_details = inspect_raw(original, source)
    result = {'source_file': source_path.name, 'source_sha256': digest(source_bytes),
              'source_revision': source['project']['revision'], 'dxf_file': dxf_path.name,
              'dxf_sha256': digest(original), 'dxf_size_bytes': len(original),
              'consumer': {'name': 'ezdxf', 'version': importlib.metadata.version('ezdxf'),
                           'read_method': 'ezdxf.readfile(original_path)', 'recover_used': False,
                           'save_or_rewrite_used': False},
              'raw_errors': raw_errors, 'raw_details': raw_details,
              'consumer_errors': [], 'audit_errors': [], 'audit_fixes': []}
    try:
        drawing = ezdxf.readfile(dxf_path)
        try:
            result['consumer_details'] = consumer_geometry(drawing, source)
        except Exception as exc:
            result['consumer_errors'].append(f'{type(exc).__name__}: {exc}')
        auditor = drawing.audit()
        def issue(item):
            return {'code': int(item.code), 'message': item.message,
                    'entity_type': item.entity.dxftype() if item.entity is not None else None,
                    'entity_handle': item.entity.dxf.get('handle') if item.entity is not None else None,
                    'data': repr(item.data)}
        result['audit_errors'] = [issue(item) for item in auditor.errors]
        result['audit_fixes'] = [issue(item) for item in auditor.fixes]
    except Exception as exc:
        result['consumer_errors'].append(f'{type(exc).__name__}: {exc}')
    result['original_bytes_preserved'] = dxf_path.read_bytes() == original
    result['accepted'] = not any(result[k] for k in ('raw_errors', 'consumer_errors', 'audit_errors', 'audit_fixes')) and result['original_bytes_preserved']
    return result


def native_snapshot_check(directory):
    result = {'accepted': False, 'errors': [], 'hashes': {}}
    try:
        expected_before = json.loads((ROOT / 'input/mixed.weftforge').read_bytes())
        expected_after = json.loads((ROOT / 'input/native-expected.weftforge').read_bytes())
        names = ['source-before.weftforge', 'source-after-cancel.weftforge', 'source-after.weftforge', 'source-after-repeat.weftforge', 'reopened.weftforge']
        data = {name: (directory / name).read_bytes() for name in names}
        result['hashes'] = {name: digest(raw) for name, raw in data.items()}
        for name in names[:2]:
            assert json.loads(data[name]) == expected_before, f'{name} source JSON differs from predeclared r20 fixture'
        for name in names[2:]:
            assert json.loads(data[name]) == expected_after, f'{name} source JSON differs from declared resolution/revision-only r21 delta'
        assert data[names[0]] == data[names[1]], 'Cancel changed saved project bytes'
        assert data[names[2]] == data[names[3]] == data[names[4]], 'Repeat export or reopen changed saved project bytes'
        sentinel = (directory / 'native-cancel-sentinel.dxf').read_bytes()
        assert sentinel == b'FILE-005 cancellation sentinel; preserve these original bytes.\n', 'Cancel destination changed'
        assert (directory / 'native-mixed.dxf').read_bytes() == (directory / 'native-mixed-repeat.dxf').read_bytes(), 'native repeated destinations differ'
        result['sentinel_sha256'] = digest(sentinel)
        result['accepted'] = True
    except Exception as exc:
        result['errors'].append(f'{type(exc).__name__}: {exc}')
    return result


def self_test():
    source = json.loads((ROOT / 'input/mixed.weftforge').read_bytes())
    expected = authored(source)
    cubic = next(e for e in expected if e['kind'] == 'SPLINE')
    cp = cubic['points']
    spline = BSpline(cp, order=4, knots=KNOTS)
    errors = [max(abs(a-b) for a,b in zip(spline.point(t), bezier(cp,t))) for t in SAMPLES]
    assert max(errors) <= TOLERANCE_MM
    independent_doc = ezdxf.new('R2000')
    independent_doc.header['$INSUNITS'] = 4
    independent_doc.header['$MEASUREMENT'] = 1
    independent_doc.header['$LUNITS'] = 2
    for layer in sorted({e['layer'] for e in expected}):
        independent_doc.layers.new(layer)
    for e in expected:
        if e['kind'] == 'LINE':
            independent_doc.modelspace().add_line(*e['points'],dxfattribs={'layer':e['layer']})
        else:
            entity = independent_doc.modelspace().add_open_spline(e['points'],degree=3,knots=KNOTS,dxfattribs={'layer':e['layer'],'extrusion':(0,0,1)})
            entity.dxf.flags = 8
    calibration_consumer = consumer_geometry(independent_doc,source)
    independent_audit = independent_doc.audit()
    assert not independent_audit.errors and not independent_audit.fixes
    witnesses = []
    def rejection(label, fn):
        try:
            fn()
        except (AssertionError, ValueError, UnicodeError, IndexError) as exc:
            witnesses.append({'name': label, 'rejected': True, 'reason': str(exc)})
        else:
            raise AssertionError(f'negative witness accepted: {label}')
    line = expected[0]
    def record_for(e):
        r = [(0,e['kind']), (5,'AA'), (330,'1'), (100,'AcDbEntity'), (8,e['layer']), (67,'0'), (410,'Model'), (370,'-1'), (100,'AcDbLine' if e['kind']=='LINE' else 'AcDbSpline')]
        if e['kind']=='LINE':
            for code,p in zip((10,11),e['points']):
                r.extend((code+offset,repr(value)) for offset,value in zip((0,10,20),p))
        else:
            r.extend((code,str(value)) for code,value in zip((70,71,72,73,74),(8,3,8,4,0)))
            r.extend((40,repr(k)) for k in KNOTS)
            for p in e['points']:
                r.extend((10+offset,repr(value)) for offset,value in zip((0,10,20),p))
            r.extend([(210,'0'),(220,'0'),(230,'1')])
        return r
    good_line, good_cubic = record_for(line), record_for(cubic)
    assert check_geometry(good_line,line) == line['points']
    assert check_geometry(good_cubic,cubic) == cp
    def alter_coords(record, transform):
        return [(c,repr(transform(c,number(v)))) if c in (10,20,30,11,21,31) else (c,v) for c,v in record]
    rejection('f32 roundoff after scale', lambda: check_geometry(alter_coords(good_cubic,lambda c,v: struct.unpack('f',struct.pack('f',v))[0]),cubic))
    rejection('metres without x1000', lambda: check_geometry(alter_coords(good_line,lambda c,v:v/1000),line))
    rejection('SVG Y inversion', lambda: check_geometry(alter_coords(good_line,lambda c,v:-v if c in (20,21) else v),line))
    rejection('3D placement translation applied', lambda: check_geometry(alter_coords(good_line,lambda c,v:v+510 if c in (10,11) else v),line))
    rejection('fixed 6-decimal output', lambda: check_geometry(alter_coords(good_cubic,lambda c,v:round(v,6)),cubic))
    rejection('cubic endpoints used as interpolating controls', lambda: check_geometry([(c,'0.0') if c==20 else (c,v) for c,v in good_cubic],cubic))
    rejection('SPLINE rational flag', lambda: check_geometry([(c,'12') if c==70 else (c,v) for c,v in good_cubic],cubic))
    rejection('SPLINE wrong knot', lambda: check_geometry([(c,'0.25') if c==40 else (c,v) for c,v in good_cubic],cubic))
    rejection('SPLINE wrong normal', lambda: check_geometry([(c,'-1') if c==230 else (c,v) for c,v in good_cubic],cubic))
    rejection('missing explicit LINE Z', lambda: check_geometry([(c,v) for c,v in good_line if c!=30],line))
    rejection('wrong piece/order', lambda: check_geometry([(c,'WF_PIECE_11') if c==8 else (c,v) for c,v in good_line],line))
    rejection('missing model-space declaration', lambda: check_geometry([(c,v) for c,v in good_line if c!=410],line))
    rejection('extra closing edge count', lambda: require(len(expected)+1==len(expected),'extra boundary entity'))
    minimum = b''.join(f'0\nSECTION\n2\n{name}\n0\nENDSEC\n'.encode() for name in SECTIONS) + b'0\nEOF\n'
    assert list(parse_raw(minimum)[0]) == SECTIONS
    rejection('missing CLASSES section', lambda: parse_raw(minimum.replace(b'0\nSECTION\n2\nCLASSES\n0\nENDSEC\n',b'')))
    rejection('BOM', lambda: parse_raw(b'\xef\xbb\xbf'+minimum))
    rejection('CRLF', lambda: parse_raw(minimum.replace(b'\n',b'\r\n')))
    rejection('missing final newline', lambda: parse_raw(minimum[:-1]))
    rejection('duplicate required field', lambda: one([(70,'8'),(70,'8')],70))
    # Hand-authored raw-tag calibration is independent of the candidate and consumer writer.
    # It exists only in self-test memory, never as an expected production export.
    calibration_sections = {name: [] for name in SECTIONS}
    calibration_sections['HEADER'] = [(9,'$ACADVER'),(1,'AC1015'),(9,'$INSUNITS'),(70,'4'),(9,'$MEASUREMENT'),(70,'1'),(9,'$LUNITS'),(70,'2'),(9,'$HANDSEED'),(5,'1000')]
    table_handles = {name: f'{i:X}' for i,name in enumerate(TABLES,0x10)}
    next_record = 0x20
    record_handles = {}
    for name in TABLES:
        names = {'LTYPE':['BYBLOCK','BYLAYER','CONTINUOUS'],'LAYER':['0','WF_PIECE_2','WF_PIECE_11'],
                 'STYLE':['STANDARD'],'APPID':['ACAD'],'DIMSTYLE':['STANDARD'],
                 'BLOCK_RECORD':['*Model_Space','*Paper_Space']}.get(name,[])
        table = [(0,'TABLE'),(2,name),(5,table_handles[name]),(330,'0'),(100,'AcDbSymbolTable'),(70,str(len(names)))]
        calibration_sections['TABLES'].extend(table)
        for record_name in names:
            rh=f'{next_record:X}'
            next_record+=1
            record_handles[name,record_name]=rh
            record=[(0,name),(105 if name=='DIMSTYLE' else 5,rh),(330,table_handles[name]),(100,'AcDbSymbolTableRecord'),(100,SUBCLASSES[name]),(2,record_name)]
            if name=='LAYER':
                record.extend([(70,'0'),(62,'7'),(6,'CONTINUOUS')])
            if name=='BLOCK_RECORD':
                record.append((340,'63' if record_name=='*Model_Space' else '64'))
            calibration_sections['TABLES'].extend(record)
        calibration_sections['TABLES'].append((0,'ENDTAB'))
    for i,name in enumerate(['*Model_Space','*Paper_Space']):
        bh=record_handles['BLOCK_RECORD',name]
        calibration_sections['BLOCKS'].extend([(0,'BLOCK'),(5,f'{0x30+i*2:X}'),(330,bh),(100,'AcDbEntity'),(8,'0'),(100,'AcDbBlockBegin'),(2,name),(70,'0'),(10,'0'),(20,'0'),(30,'0'),(0,'ENDBLK'),(5,f'{0x31+i*2:X}'),(330,bh),(100,'AcDbEntity'),(8,'0'),(100,'AcDbBlockEnd')])
    for i,wanted in enumerate(expected):
        record=record_for(wanted)
        record=[(5,f'{0x40+i:X}') if c==5 else (330,record_handles['BLOCK_RECORD','*Model_Space']) if c==330 else (c,v) for c,v in record]
        calibration_sections['ENTITIES'].extend(record)
    calibration_sections['OBJECTS']=[(0,'DICTIONARY'),(5,'60'),(330,'0'),(100,'AcDbDictionary'),(3,'ACAD_GROUP'),(350,'61'),(3,'ACAD_LAYOUT'),(350,'62'),(0,'DICTIONARY'),(5,'61'),(330,'60'),(100,'AcDbDictionary'),(0,'DICTIONARY'),(5,'62'),(330,'60'),(100,'AcDbDictionary'),(3,'Model'),(350,'63'),(3,'Layout1'),(350,'64')]
    for i,(name,bname) in enumerate([('Model','*Model_Space'),('Layout1','*Paper_Space')]):
        calibration_sections['OBJECTS'].extend([(0,'LAYOUT'),(5,f'{0x63+i:X}'),(330,'62'),(100,'AcDbPlotSettings'),(100,'AcDbLayout'),(1,name),(71,str(i)),(330,record_handles['BLOCK_RECORD',bname])])
    def raw_calibration(changes=None):
        parts=[(999,'WeftForge FILE-005-v1 source document revision 20; millimetres; UV-to-XY; pattern boundaries only')]
        sections=deepcopy(calibration_sections)
        if changes:
            changes(sections)
        for name in SECTIONS:
            parts.extend([(0,'SECTION'),(2,name),*sections[name],(0,'ENDSEC')])
        parts.append((0,'EOF'))
        return ''.join(f'{c}\n{v}\n' for c,v in parts).encode('ascii')
    calibration_errors,calibration_details=inspect_raw(raw_calibration(),source)
    assert not calibration_errors, calibration_errors
    assert calibration_details['handle_count'] > 0
    def raw_rejection(label,change,expected_error):
        errors,_=inspect_raw(raw_calibration(change),source)
        assert errors and any(expected_error in error for error in errors), (label,errors)
        witnesses.append({'name':label,'rejected':True,'raw_errors':errors})
    def replace_tag(section,old,new):
        section[section.index(old)]=new
    raw_rejection('missing raw INSUNITS despite consumer defaults',lambda s: s['HEADER'].__setitem__(slice(2,4),[]),'$INSUNITS')
    raw_rejection('raw TABLE count mismatch',lambda s: replace_tag(s['TABLES'],(70,'0'),(70,'1')),'count mismatch')
    raw_rejection('duplicate raw handle',lambda s: replace_tag(s['ENTITIES'],(5,'41'),(5,'40')),'duplicate handle')
    raw_rejection('wrong model-space owner',lambda s: replace_tag(s['ENTITIES'],(330,record_handles['BLOCK_RECORD','*Model_Space']),(330,'60')),'wrong model-space owner')
    raw_rejection('unresolved raw reference',lambda s: s['ENTITIES'].append((340,'FFFF')),'unresolved group 340')
    raw_rejection('missing raw BLOCK_RECORD layout link',lambda s: s['TABLES'].remove((340,'63')),'OBJECTS/layout links')
    raw_rejection('missing raw LAYOUT reverse block link',lambda s: s['OBJECTS'].remove((330,record_handles['BLOCK_RECORD','*Model_Space'])),'OBJECTS/layout links')
    raw_rejection('missing raw ACAD_GROUP declaration',lambda s: s['OBJECTS'].__setitem__(slice(4,6),[]),'root entries')
    raw_rejection('HANDSEED below emitted handles',lambda s: replace_tag(s['HEADER'],(5,'1000'),(5,'1')),'HANDSEED')
    raw_rejection('DIMSTYLE group 5 instead of 105',lambda s: replace_tag(s['TABLES'],(105,record_handles['DIMSTYLE','STANDARD']),(5,record_handles['DIMSTYLE','STANDARD'])),'DIMSTYLE handle')
    raw_rejection('R2007 BLOCK_RECORD insertion units in R2000',lambda s: s['TABLES'].insert(s['TABLES'].index((340,'63')),(70,'4')),'R2007 BLOCK_RECORD')
    raw_rejection('extra raw closing LINE entity',lambda s: s['ENTITIES'].extend(record_for(line)),'entity count')
    raw_rejection('missing raw table-record owner',lambda s: s['TABLES'].remove((330,table_handles['LAYER'])),'TABLES')

    rejection('nonfinite numeric data', lambda: number('nan'))
    rejection('invalid handle', lambda: handle('x123'))
    # Geometry perturbations must also fail sampled shape comparison by a material amount.
    modified = deepcopy(cp)
    modified[1] = (modified[1][0]+0.001,modified[1][1],0.0)
    bad = BSpline(modified,order=4,knots=KNOTS)
    bad_error = max(abs(a-b) for t in SAMPLES for a,b in zip(bad.point(t),bezier(cp,t)))
    assert bad_error > 1000*TOLERANCE_MM
    witnesses.append({'name':'0.001 mm cubic control perturbation','rejected':True,'max_sample_error_mm':bad_error})
    # Uniform XY translation must not accidentally agree with a cubic sample at symmetric t.
    assert cp[1] != cp[2] and cp[1][0] != cp[2][0]
    return {'accepted':True,'candidate_outputs_read':False,'positive_spline_max_sample_error_mm':max(errors),
            'negative_witness_count':len(witnesses),'negative_witnesses':witnesses}


def main():
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest='command',required=True)
    test = sub.add_parser('self-test')
    test.add_argument('--report',type=Path,required=True)
    check = sub.add_parser('check')
    check.add_argument('--source',type=Path,required=True)
    check.add_argument('--dxf',type=Path,required=True)
    check.add_argument('--report',type=Path,required=True)
    native = sub.add_parser('native-snapshots')
    native.add_argument('--directory',type=Path,required=True)
    native.add_argument('--report',type=Path,required=True)
    args=parser.parse_args()
    if args.command=='self-test':
        result=self_test()
    elif args.command=='check':
        result=evaluate(args.source,args.dxf)
    else:
        result=native_snapshot_check(args.directory)
    result['evaluator_sha256']=digest(Path(__file__).read_bytes())
    result['contract']='FILE-005-v1'
    result['sample_t']=SAMPLES
    result['sample_tolerance_mm']=TOLERANCE_MM
    args.report.write_text(json.dumps(result,indent=2,ensure_ascii=False,allow_nan=False)+'\n',encoding='utf-8')
    print(json.dumps({'accepted':result['accepted'],'report':str(args.report),
                      'raw_errors':result.get('raw_errors',[]),'consumer_errors':result.get('consumer_errors',[]),
                      'audit_errors':result.get('audit_errors',[]),'audit_fixes':result.get('audit_fixes',[])}))
    return 0 if result['accepted'] else 1


if __name__=='__main__':
    sys.exit(main())
