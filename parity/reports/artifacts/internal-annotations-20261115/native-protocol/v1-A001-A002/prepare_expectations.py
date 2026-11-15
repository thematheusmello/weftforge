#!/usr/bin/env python3
"""Original saved-state models; no product imports, subprocesses or app execution."""
import copy
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
ORIGINAL = ROOT.parent / 'internal-line-independent-v2-20261115'


def write(name, data):
    target = ROOT / name
    target.write_text(json.dumps(data, indent=2, allow_nan=False) + '\n')


def revision(doc, value):
    result = copy.deepcopy(doc)
    result['project']['revision'] = value
    return result


def item(doc, collection, identity):
    return next(x for x in doc['project'][collection] if x['id'] == identity)


def line(doc, start, end, owner=2):
    result = copy.deepcopy(doc)
    project = result['project']
    first = project['next_id']
    project['points'] += [{'id': first, 'position': start}, {'id': first + 1, 'position': end}]
    project['curves'].append({'id': first + 2, 'start': first, 'end': first + 1, 'controls': None})
    item(result, 'pieces', owner)['internal_curves'].append(first + 2)
    project['next_id'] = first + 3
    project['revision'] += 1
    return result


def main():
    # Public B657 default constructor constants, not a CLI-produced empty project.
    material = {
        'id': 1, 'name': 'Cotton approximation (uncalibrated)',
        'areal_density': .18, 'warp_stiffness': 400., 'weft_stiffness': 400.,
        'shear_stiffness': 100., 'bending_stiffness': .003,
        'damping': .2, 'friction': .4, 'collision_thickness': .003,
        'color': [.22, .55, .76],
    }
    capsules = [
        {'a': [0., .9, 0.], 'b': [0., 1.4, 0.], 'radius': .18},
        {'a': [0., 1.53, 0.], 'b': [0., 1.65, 0.], 'radius': .105},
        {'a': [-.22, 1.4, 0.], 'b': [-.55, 1.22, 0.], 'radius': .065},
        {'a': [.22, 1.4, 0.], 'b': [.55, 1.22, 0.], 'radius': .065},
        {'a': [-.09, .88, 0.], 'b': [-.09, .18, 0.], 'radius': .085},
        {'a': [.09, .88, 0.], 'b': [.09, .18, 0.], 'radius': .085},
    ]
    empty = {'format': 'weftforge', 'version': 4, 'project': {
        'revision': 0, 'points': [], 'curves': [], 'pieces': [],
        'materials': [material], 'seams': [], 'sewing_groups': [],
        'mannequin': {'capsules': capsules}, 'resolution': .07, 'next_id': 2,
    }}
    sources = []
    fixtures = {}
    for name in ('R', 'R_line', 'L', 'C', 'S_line'):
        source = ORIGINAL / 'fixtures' / (name + '.weftforge')
        raw = source.read_bytes()
        fixtures[name] = json.loads(raw)
        (ROOT / 'fixtures' / source.name).write_bytes(raw)
        sources.append({'original': str(source.relative_to(ROOT.parent)),
                        'copy': 'fixtures/' + source.name, 'sha256': hashlib.sha256(raw).hexdigest()})
    square = copy.deepcopy(fixtures['R'])
    item(square, 'pieces', 2)['name'] = 'Pattern 1'
    assert square['project']['materials'] == empty['project']['materials']
    assert square['project']['mannequin'] == empty['project']['mannequin']
    created = line(square, [.125, .25], [.875, .75])
    edited = revision(created, 3)
    item(edited, 'points', 11)['position'] = [.25, .375]
    item(edited, 'points', 12)['position'] = [.75, .625]
    item(edited, 'pieces', 2)['grain_angle'] = .47
    states = {
        'A': {'A00-empty': empty, 'A01-square': square, 'A02-created': created,
              'A03-edited': edited, 'A04-undo': revision(created, 4),
              'A05-redo': revision(edited, 5)},
    }
    restart = revision(edited, 6)
    item(restart, 'points', 12)['position'][1] = .6875
    wide = revision(restart, 9)
    for point in wide['project']['points']:
        point['position'][0] *= 2.
    resized = revision(wide, 10)
    for point in resized['project']['points']:
        point['position'][1] /= 2.
    turned = revision(resized, 11)
    # Exact quarter-turn geometry; represented placement/grain are graded independently.
    for point in turned['project']['points']:
        u, v = point['position']
        point['position'] = [-v, u]
    no_line = revision(turned, 12)
    no_line['project']['points'] = [p for p in no_line['project']['points'] if p['id'] not in (11, 12)]
    no_line['project']['curves'] = [c for c in no_line['project']['curves'] if c['id'] != 13]
    item(no_line, 'pieces', 2)['internal_curves'] = []
    no_owner = revision(turned, 14)
    for key in ('points', 'curves', 'pieces'):
        no_owner['project'][key] = []
    states['B'] = {'B06-reedited': restart, 'B07-undo': revision(edited, 7),
                   'B08-redo': revision(restart, 8), 'B09-wide': wide, 'B10-resized': resized,
                   'B11-turned': turned, 'B12-line-removed': no_line,
                   'B14-owner-removed': no_owner, 'B15-owner-restored': revision(turned, 15)}
    coincident = line(fixtures['S_line'], [.125, .25], [.5, .5])
    identity = revision(coincident, 7)
    item(identity, 'points', 25)['position'] = [.25, .375]
    combined = revision(identity, 8)
    item(combined, 'curves', 7)['controls'][1][0] = .25
    item(combined, 'points', 23)['position'] = [.75, .625]
    states['C'] = {'C07-selected-identity': identity, 'C08-control-and-drag': combined,
                   'C09-undo': revision(identity, 9), 'C10-redo': revision(combined, 10)}
    post_io = line(fixtures['R_line'], [.25, .25], [.75, .75])
    states['D'] = {'D03-after-io': post_io, 'D00-new-empty': copy.deepcopy(empty),
                   'D05-recovered': copy.deepcopy(fixtures['S_line'])}
    accepted_l = line(fixtures['L'], [.2, .8], [.2, .2])
    accepted_c = line(fixtures['C'], [.1, .75], [.9, .75])
    states['E'] = {'E02-L-accepted': accepted_l, 'E03-C-accepted': accepted_c}
    # Native modal/file parsing failures are original deliberately malformed bytes.
    (ROOT / 'fixtures/S_line.weftforge.recovery').write_bytes((ROOT / 'fixtures/S_line.weftforge').read_bytes())
    malformed = copy.deepcopy(fixtures['R_line'])
    del malformed['project']['pieces'][0]['internal_curves']
    write('fixtures/malformed-v4-missing-internals.weftforge', malformed)
    (ROOT / 'fixtures/invalid-save.weftforge').write_bytes(b'Original invalid native destination; preserve these bytes.\n')
    (ROOT / 'fixtures/invalid-save.weftforge.recovery').write_bytes(b'Original prior recovery; preserve these bytes.\n')
    write('EXPECTATIONS.json', {
        'status': 'draft_prospective_original_expectations_not_native_evidence',
        'default_basis': 'B657 document.rs constructors; exactly six analytic mannequin capsules',
        'route_basis': '703a main.rs; pending later Escape/live-text fix is not bound',
        'native_color_policy': 'Material.color is f32: compare f32 bits, not widened f64 wire bits',
        'states': states,
        'previous_saved': {case: {label: (list(values)[index - 1] if case in 'ABC' and index else None)
                                  for index, label in enumerate(values)} for case, values in states.items()},
        'primary_paths': {case: {label: ('work/' + case + '.weftforge' if case in 'ABC' else
                                        'work/' + label + '.weftforge') for label in values}
                          for case, values in states.items()},
        'transform': {'case': 'B', 'from_state': 'B10-resized', 'affected_states': [
            'B11-turned', 'B12-line-removed', 'B15-owner-restored'],
            'angle_rad': 1.5707963267948966, 'pivot_m': [0., 0.], 'translation_m': [0., 0.],
            'geometry_tolerance_m': 1e-10, 'orientation_tolerance': 1e-12,
            'grain_tolerance': 1e-12, 'world_landmark_tolerance_m': 1e-10},
        'exports': {'case': 'E', 'baseline_revision': 2, 'annotated_revision': 3,
            'names': ['C-before.svg', 'C-after.svg', 'C-before.dxf', 'C-after.dxf',
                      'C-before.obj', 'C-after.obj'],
            'policy': 'Actual SVG/OBJ raw bytes equal; DXF validate exactly one own-revision group999 comment, normalize only its value. No OBJ line primitives.'},
    })
    write('ORIGINAL-INPUTS.json', sources)
    print(json.dumps({'status': 'prepared_original_models', 'native_executed': False,
                      'source_or_binary_bound': False, 'saved_state_models': sum(map(len, states.values()))}))


if __name__ == '__main__':
    main()
