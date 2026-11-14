"""Review saved cloth outputs without running a simulator or importing its analyzer.

Usage: python3 replay_review.py BUNDLE [--source CHECKOUT] [--historical BUNDLE]
Writes a JSON receipt to stdout. All input paths in that receipt are relative.
"""
import argparse
from collections import defaultdict
import copy
import hashlib
import json
import math
from pathlib import Path
import struct
import subprocess


def sub(a, b):
    return [x - y for x, y in zip(a, b)]


def dot(a, b):
    return sum(x * y for x, y in zip(a, b))


def scale(a, t):
    return [x * t for x in a]


def cross(a, b):
    return [a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]


def length(a):
    return math.sqrt(dot(a, a))


def near(a, b, tolerance=2e-11):
    assert math.isfinite(a) and math.isfinite(b) and abs(a - b) <= tolerance, (a, b)
    return abs(a - b)


def file_hash(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def read(path):
    return json.loads(path.read_text())


def lerp(a, b, t):
    return [x + t * (y - x) for x, y in zip(a, b)]


def stitch_point(endpoint, positions):
    return lerp(*(positions[i] for i in endpoint['vertices']), endpoint['weight'])


def segment_closest(p, a, b):
    edge = sub(b, a)
    t = max(0., min(1., dot(sub(p, a), edge) / dot(edge, edge))) if dot(edge, edge) else 0.
    return lerp(a, b, t), t


def triangle_distance(p, vertices):
    a, b, c = vertices
    ab, ac = sub(b, a), sub(c, a)
    normal = cross(ab, ac)
    if length(normal) <= 1e-14:
        return None
    ap = sub(p, a)
    determinant = dot(ab, ab) * dot(ac, ac) - dot(ab, ac) ** 2
    assert determinant > 0.
    u = (dot(ap, ab) * dot(ac, ac) - dot(ap, ac) * dot(ab, ac)) / determinant
    v = (dot(ap, ac) * dot(ab, ab) - dot(ap, ab) * dot(ab, ac)) / determinant
    if u >= 0. and v >= 0. and u + v <= 1.:
        return abs(dot(ap, normal)) / length(normal)
    return min(length(sub(p, segment_closest(p, x, y)[0]))
               for x, y in [(a, b), (b, c), (c, a)])


def triangle_data(meta):
    pieces = {p['id']: p for p in meta['project']['pieces']}
    materials = {m['id']: m for m in meta['project']['materials']}
    data, mass, thickness, neighbors, edges = [], [0.] * len(meta['uv_m']), [0.] * len(meta['uv_m']), [set() for _ in meta['uv_m']], defaultdict(list)
    angles = []
    for index, tri in enumerate(meta['triangles']):
        assert len(set(meta['piece_ids'][i] for i in tri)) == 1
        piece = pieces[meta['piece_ids'][tri[0]]]
        material = materials[piece['material']]
        a, b, c = [meta['uv_m'][i] for i in tri]
        ab, ac = sub(b, a), sub(c, a)
        det = ab[0] * ac[1] - ab[1] * ac[0]
        assert abs(det) > 1e-12
        area = abs(det) / 2
        inverse = [[ac[1] / det, -ac[0] / det], [-ab[1] / det, ab[0] / det]]
        # The inverse rows map world edges to material U/V derivatives.
        gradients = [[-(inverse[0][0] + inverse[1][0]), -(inverse[0][1] + inverse[1][1])], inverse[0], inverse[1]]
        co, si = math.cos(piece['grain_angle']), math.sin(piece['grain_angle'])
        coefficients = [[co * u + si * v, -si * u + co * v] for u, v in gradients]
        data.append((tri, piece, material, area, coefficients))
        for i in tri:
            mass[i] += area * material['areal_density'] / 3
            thickness[i] = max(thickness[i], material['collision_thickness'])
            neighbors[i].update(tri)
        for j in range(3):
            edge = tuple(sorted([tri[j], tri[(j + 1) % 3]]))
            edges[edge].append((index, tri[(j + 2) % 3]))
            q = [meta['uv_m'][tri[k]] for k in [j, (j + 1) % 3, (j + 2) % 3]]
            x, y = sub(q[1], q[0]), sub(q[2], q[0])
            angles.append(math.degrees(math.acos(max(-1., min(1., dot(x, y) / length(x) / length(y))))))
    for stitch in meta['stitches']:
        for a in stitch['a']['vertices']:
            for b in stitch['b']['vertices']:
                neighbors[a].add(b)
                neighbors[b].add(a)
    return data, mass, thickness, neighbors, edges, [min(angles), max(angles)]


def membrane(data, positions):
    energy, force, peak, signed_values, per_piece = [0., 0., 0.], [[0., 0., 0.] for _ in positions], None, [], {}
    for index, (tri, piece, material, area, coefficients) in enumerate(data):
        relative = [sub(positions[i], positions[tri[0]]) for i in tri]
        axes = [[sum(relative[j][k] * coefficients[j][axis] for j in range(3)) for k in range(3)] for axis in range(2)]
        signed = [length(axis) - 1. for axis in axes]
        for axis, value in enumerate(signed):
            item = {'triangle': index, 'piece': piece['id'], 'vertices': tri, 'axis': ['warp', 'weft'][axis], 'signed_strain': value, 'absolute_strain': abs(value)}
            if peak is None or abs(value) > peak['absolute_strain']:
                peak = item
            if piece['id'] not in per_piece or abs(value) > per_piece[piece['id']]['absolute_strain']:
                per_piece[piece['id']] = item
            signed_values.append(value)
        strains = [(dot(axes[0], axes[0]) - 1.) / 2, (dot(axes[1], axes[1]) - 1.) / 2, dot(*axes)]
        stiffness = [material[k] for k in ['warp_stiffness', 'weft_stiffness', 'shear_stiffness']]
        for k in range(3):
            energy[k] += area * stiffness[k] * strains[k] ** 2 / 2
        for i, (cw, cf) in zip(tri, coefficients):
            for k in range(3):
                force[i][k] -= area * (stiffness[0] * strains[0] * axes[0][k] * cw
                                      + stiffness[1] * strains[1] * axes[1][k] * cf
                                      + stiffness[2] * strains[2] * (axes[1][k] * cw + axes[0][k] * cf))
    return {'peak': peak, 'minimum_signed': min(signed_values), 'maximum_signed': max(signed_values), 'energy_components_J': energy, 'energy_J': sum(energy), 'per_piece': per_piece}, force


def dihedral(x):
    a, b, c, d = x
    e = sub(b, a)
    n1, n2 = cross(e, sub(c, a)), cross(sub(d, a), e)
    assert length(e) >= 1e-12 and length(n1) >= 1e-14 and length(n2) >= 1e-14
    return math.atan2(dot(cross(n1, n2), e) / length(e), dot(n1, n2))


def bending_data(meta, data, edges):
    result = []
    for (a, b), faces in sorted(edges.items()):
        assert len(faces) <= 2
        if len(faces) != 2:
            continue
        (ti, c), (tj, d) = faces
        rigidity = (data[ti][2]['bending_stiffness'] + data[tj][2]['bending_stiffness']) / 2
        if not rigidity:
            continue
        ids = [a, b, c, d]
        x = [meta['rest_positions_m'][i] for i in ids]
        e = sub(x[1], x[0])
        mean_height = (length(cross(e, sub(x[2], x[0]))) + length(cross(e, sub(x[3], x[0])))) / (2 * length(e))
        result.append((ids, dihedral(x), rigidity * length(e) / mean_height))
    return result


def bend_energy(x, angle, stiffness):
    delta = dihedral(x) - angle
    delta = math.atan2(math.sin(delta), math.cos(delta))
    return stiffness * delta * delta / 2


def bending(bends, positions, forces):
    total, force = 0., [[0., 0., 0.] for _ in positions]
    epsilon = 1e-6
    for ids, rest, stiffness in bends:
        x = [list(positions[i]) for i in ids]
        total += bend_energy(x, rest, stiffness)
        if forces:
            for j, node in enumerate(ids):
                for k in range(3):
                    original = x[j][k]
                    x[j][k] = original + epsilon
                    plus = bend_energy(x, rest, stiffness)
                    x[j][k] = original - epsilon
                    minus = bend_energy(x, rest, stiffness)
                    x[j][k] = original
                    force[node][k] -= (plus - minus) / (2 * epsilon)
    return total, force


def seams(meta, positions, targets):
    gaps, energy, force = [], 0., [[0., 0., 0.] for _ in positions]
    for stitch, target in zip(meta['stitches'], targets):
        delta = sub(stitch_point(stitch['a'], positions), stitch_point(stitch['b'], positions))
        gap = length(delta)
        gaps.append(gap)
        compliance = stitch['compliance_m_per_N']
        energy += (gap - target) ** 2 / (2 * compliance)
        if gap > 1e-14:
            for end, sign in [('a', 1), ('b', -1)]:
                endpoint = stitch[end]
                for node, weight in zip(endpoint['vertices'], [1 - endpoint['weight'], endpoint['weight']]):
                    for k in range(3):
                        force[node][k] -= sign * weight * delta[k] * (gap - target) / (compliance * gap)
    return max(gaps, default=0.), energy, force


def analytic_contact(meta, positions, thickness):
    maximum, clearances = 0., []
    for node, (p, offset) in enumerate(zip(positions, thickness)):
        maximum = max(maximum, offset - p[1])
        nearest = None
        for index, capsule in enumerate(meta['project']['mannequin']['capsules']):
            closest, fraction = segment_closest(p, capsule['a'], capsule['b'])
            clearance = length(sub(p, closest)) - capsule['radius'] - offset
            maximum = max(maximum, -clearance)
            item = {'node': node, 'capsule': index, 'clearance_m': clearance, 'axial_fraction': fraction}
            if nearest is None or clearance < nearest['clearance_m']:
                nearest = item
        clearances.append(nearest)
    return maximum, clearances


def cloth_contact(meta, positions, thickness, excluded):
    maximum, worst, pairs = 0., None, 0
    for node, p in enumerate(positions):
        for index, tri in enumerate(meta['triangles']):
            if excluded[node].intersection(tri):
                continue
            pairs += 1
            distance = triangle_distance(p, [positions[i] for i in tri])
            if distance is None:
                continue
            depth = thickness[node] + max(thickness[i] for i in tri) - distance
            if depth > maximum:
                maximum, worst = depth, {'vertex': node, 'triangle': index}
    return {'max_supported_static_cloth_penetration_m': maximum, 'worst_pair': worst, 'eligible_vertex_triangle_pairs': pairs}


def motion(mass, positions, velocity):
    total = sum(mass)
    com = [sum(m * p[k] for m, p in zip(mass, positions)) / total for k in range(3)]
    momentum = [sum(m * v[k] for m, v in zip(mass, velocity)) for k in range(3)]
    kinetic = sum(m * dot(v, v) / 2 for m, v in zip(mass, velocity))
    angular = [0., 0., 0.]
    for m, p, v in zip(mass, positions, velocity):
        term = scale(cross(sub(p, com), v), m)
        angular = [a + b for a, b in zip(angular, term)]
    return {'mass_kg': total, 'center_of_mass_m': com, 'momentum_kg_m_per_s': momentum,
            'angular_momentum_about_com_kg_m2_per_s': angular, 'kinetic_energy_J': kinetic,
            'rms_speed_m_per_s': math.sqrt(2 * kinetic / total), 'max_speed_m_per_s': max(map(length, velocity))}


def drift(a, b, mass):
    squares = [dot(sub(x, y), sub(x, y)) for x, y in zip(a, b)]
    return {'max_m': math.sqrt(max(squares)), 'mass_weighted_rms_m': math.sqrt(sum(m * q for m, q in zip(mass, squares)) / sum(mass))}


def self_check():
    meta = {'project': {'pieces': [{'id': 1, 'material': 2, 'grain_angle': 0.}],
                        'materials': [{'id': 2, 'areal_density': .2, 'collision_thickness': .003, 'warp_stiffness': 50., 'weft_stiffness': 1000., 'shear_stiffness': 100.}]},
            'piece_ids': [1, 1, 1], 'triangles': [[0, 1, 2]], 'uv_m': [[0., 0.], [2., .1], [.3, 1.]], 'stitches': []}
    data, mass, *_ = triangle_data(meta)
    x = [[0., 0., 0.], [2.4, .1, 0.], [.36, 1., 0.]]
    values, force = membrane(data, x)
    near(values['peak']['absolute_strain'], .2, 1e-14)
    near(sum(mass), .197, 1e-14)
    for i in range(3):
        for k in range(3):
            x[i][k] += 1e-6
            ep = membrane(data, x)[0]['energy_J']
            x[i][k] -= 2e-6
            em = membrane(data, x)[0]['energy_J']
            x[i][k] += 1e-6
            near((ep - em) / 2e-6, -force[i][k], 1e-7)
    near(triangle_distance([.2, .2, 2.], [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]]), 2.)
    near(triangle_distance([2., 2., 0.], [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]]), math.sqrt(4.5))
    near(triangle_distance([-1., 0., 0.], [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]]), 1.)
    near(drift([[2., 0., 0.], [0., 0., 0.]], [[0., 0., 0.], [0., 0., 0.]], [1., 3.])['mass_weighted_rms_m'], 1.)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('bundle', type=Path)
    parser.add_argument('--source', type=Path)
    parser.add_argument('--historical', type=Path)
    args = parser.parse_args()
    self_check()
    base = args.bundle
    inputs = {}

    def load(name):
        inputs[name] = file_hash(base / name)
        return read(base / name)

    plan, erratum, probes, manifest = [load(p) for p in ['plan.json', 'plan-erratum-001.json', 'plan-probes-001.json', 'source-manifest.json']]
    source_commit = '3eddef067c8a1f03f192c8e6822a8da2a94f9bf9'
    assert plan['source_commit'] == probes['source_commit'] == manifest['commit'] == source_commit
    assert plan['source_manifest_sha256'] == probes['source_manifest_sha256'] == inputs['source-manifest.json']
    assert erratum['prior_plan_sha256'] == probes['prior_plan_sha256'] == inputs['plan.json']
    assert probes['baseline_analysis_sha256'] == file_hash(base / 'control-analysis.json')
    assert plan['unchanged_quality_limits']['max_absolute_material_axis_length_strain_below'] == .15
    assert plan['unchanged_quality_limits']['max_seam_gap_m_below'] == .005
    assert plan['unchanged_quality_limits']['max_supported_contact_penetration_m_below'] == .002
    frozen = dict(plan['frozen_code_sha256'])
    frozen.update(erratum['frozen_code_sha256'])
    frozen.update(probes['frozen_code_sha256'])
    for name, expected in frozen.items():
        assert file_hash(base / name) == expected, name
        inputs[name] = expected
    binaries = {}
    for name, expected in [(plan['executed_diagnostic_binary'], plan['diagnostic_binary_sha256']),
                           (probes['executed_probe_binary'], probes['executed_probe_binary_sha256'])]:
        binaries[name] = 'not supplied'
        if (base / name).is_file():
            assert file_hash(base / name) == expected
            binaries[name] = expected
    fixture = load('original-grain-shirt.weftforge')
    assert inputs['original-grain-shirt.weftforge'] == plan['fixture_sha256'] == probes['fixture_sha256']
    source_check = {'status': 'not supplied'}
    if args.source:
        for name, expected in manifest['files'].items():
            assert file_hash(args.source / name) == expected, name
        actual_head = subprocess.check_output(['git', '-C', str(args.source), 'rev-parse', 'HEAD'], text=True).strip()
        assert actual_head == source_commit
        assert subprocess.check_output(['git', '-C', str(args.source), 'status', '--porcelain'], text=True) == ''
        source_check = {'status': 'all recorded source hashes and clean Git HEAD verified', 'file_count': len(manifest['files']), 'head': actual_head}
    reports = {'control': load('control-analysis.json')}
    probe_report = load('probes-analysis.json')
    producer_forces = {(r.get('case', 'control'), r['frame']): r for name in ['control-independent-forces.json', 'probes-independent-forces.json'] for r in load(name)}
    cases, states_by_case, meta_by_case, all_rows, force_errors = {}, {}, {}, {}, {}
    for name, frames in [('control', 180), ('zero-g-balance', 180), ('no-avatar', 60)]:
        meta = load(f'{name}/rest-metadata.json')
        meta_by_case[name] = meta
        inputs[f'{name}/trajectory.jsonl'] = file_hash(base / name / 'trajectory.jsonl')
        states = [json.loads(row) for row in (base / name / 'trajectory.jsonl').read_text().splitlines()]
        states_by_case[name] = states
        inputs[f'{name}/steps.jsonl'] = file_hash(base / name / 'steps.jsonl')
        steps = [json.loads(row) for row in (base / name / 'steps.jsonl').read_text().splitlines()]
        completed, job = load(f'{name}/completed-case.json'), load(f'{name}-job.json')
        assert job['status'] == 'completed' and job['exit_code'] == 0 and not job.get('timed_out', False)
        assert plan['declared_at'] < job['started_at']
        if name != 'control':
            assert probes['declared_at'] < job['started_at']
        assert len(states) == frames + 1 and len(steps) == completed['completed_frames'] == frames
        assert [s['global_frame'] for s in states] == list(range(frames + 1))
        assert states[0]['positions_m'] == meta['rest_positions_m']
        assert all(v == 0 for p in states[0]['velocities_m_per_s'] for v in p)
        data, mass, thickness, excluded, edges, angles = triangle_data(meta)
        bends = bending_data(meta, data, edges)
        mass_error = max(near(a, b, 1e-14) for a, b in zip(mass, meta['masses_kg']))
        initial = [length(sub(stitch_point(s['a'], meta['rest_positions_m']), stitch_point(s['b'], meta['rest_positions_m']))) for s in meta['stitches']]
        targets, rows, maxima = list(initial), [], defaultdict(float)
        expected_samples = reports['control']['samples'] if name == 'control' else probe_report['cases'][name]['samples']
        assert len(expected_samples) == len(states)
        for frame, (state, recorded) in enumerate(zip(states, expected_samples)):
            settings, pos, vel = state['settings_last_step'], state['positions_m'], state['velocities_m_per_s']
            assert len(pos) == len(vel) == len(mass)
            assert all(math.isfinite(v) for p in pos + vel for v in p)
            assert settings == {'dt': 1 / 60, 'substeps': 8, 'iterations': 32, 'gravity': [0., -9.81, 0.] if name == 'control' and frame > 60 else [0., 0., 0.], 'seam_closure_speed': 1.}
            if frame:
                assert steps[frame - 1]['settings'] == settings
                assert steps[frame - 1]['diagnostics'] == state['diagnostics']
                for _ in range(8):
                    targets = [t - min(t, 1 / 60 / 8) for t in targets]
            near(max(targets), state['diagnostics']['seam_assembly_remaining'], 1e-14)
            values, fm = membrane(data, pos)
            eb, fb = bending(bends, pos, (name, frame) in producer_forces)
            gap, es, fs = seams(meta, pos, targets)
            penetration, clearances = analytic_contact(meta, pos, thickness)
            dynamic = motion(mass, pos, vel)
            maxima['strain_vs_diagnostics'] = max(maxima['strain_vs_diagnostics'], near(values['peak']['absolute_strain'], state['diagnostics']['max_stretch_error']))
            maxima['strain_vs_report'] = max(maxima['strain_vs_report'], near(values['peak']['absolute_strain'], recorded['strain']['peak']['absolute_strain']))
            maxima['membrane_energy_vs_report_J'] = max(maxima['membrane_energy_vs_report_J'], near(values['energy_J'], recorded['strain']['membrane_energy_J']['total']))
            maxima['bending_energy_vs_report_J'] = max(maxima['bending_energy_vs_report_J'], near(eb, recorded['bending_energy_J']))
            maxima['sewing_energy_vs_report_J'] = max(maxima['sewing_energy_vs_report_J'], near(es, recorded['temporary_sewing_energy_J'], 1e-9))
            maxima['seam_vs_diagnostics_m'] = max(maxima['seam_vs_diagnostics_m'], near(gap, state['diagnostics']['max_seam_error']))
            maxima['analytic_penetration_vs_report_m'] = max(maxima['analytic_penetration_vs_report_m'], near(penetration, recorded['independent_supported_residuals']['avatar_ground_penetration_m']))
            for key, value in dynamic.items():
                other = recorded['motion'][key]
                for a, b in zip(value if isinstance(value, list) else [value], other if isinstance(other, list) else [other]):
                    near(a, b)
            near(dynamic['kinetic_energy_J'], state['diagnostics']['kinetic_energy'])
            screen = state['diagnostics']['finite'] and not max(targets) and values['peak']['absolute_strain'] < .15 and gap < .005 and state['diagnostics']['max_contact_error'] < .002
            row = {'frame': frame, 'time_s': frame / 60, 'material': values, 'bending_energy_J': eb, 'elastic_energy_J': values['energy_J'] + eb, 'sewing_energy_J': es, 'seam_gap_m': gap, 'analytic_penetration_m': penetration, 'motion': dynamic,
                   'arithmetic_screen': screen, 'original_quality': None if name == 'no-avatar' else screen}
            if frame in [0, 24, 27, 60, frames]:
                row['static_cloth_contact'] = cloth_contact(meta, pos, thickness, excluded)
                supported = max(penetration, row['static_cloth_contact']['max_supported_static_cloth_penetration_m'])
                near(supported, state['diagnostics']['max_contact_error'], 1e-10)
                row['peak_node_capsule_clearances'] = [clearances[i] for i in values['peak']['vertices']]
            if frame:
                row['last_frame_drift_m'] = drift(pos, states[frame - 1]['positions_m'], mass)
                impulse = [scale(sub(a, b), 60.) for a, b in zip(vel, states[frame - 1]['velocities_m_per_s'])]
                row['interval_nodal_effective_force_max_N'] = max(m * length(a) for m, a in zip(mass, impulse))
                row['interval_mass_rms_acceleration_m_per_s2'] = math.sqrt(sum(m * dot(a, a) for m, a in zip(mass, impulse)) / sum(mass))
            if (name, frame) in producer_forces:
                saved = producer_forces[(name, frame)]
                combined = [[a + b + c for a, b, c in zip(f0, f1, f2)] for f0, f1, f2 in zip(fm, fb, fs)]
                errors = {}
                for key, forces in [('membrane_force_N', fm), ('bending_force_N', fb), ('sewing_force_N', fs), ('combined_conservative_force_N', combined)]:
                    errors[key] = max(near(x, y, 5e-8) for a, b in zip(forces, saved[key]) for x, y in zip(a, b))
                force_errors[f'{name}/{frame}'] = errors
                row['conservative_force_maxima_N'] = {key: max(map(length, forces)) for key, forces in [('membrane', fm), ('bending', fb), ('sewing', fs), ('combined', combined)]}
            rows.append(row)
        assert completed['final_diagnostics'] == states[-1]['diagnostics']
        all_rows[name] = rows
        cases[name] = {'frames': frames, 'vertices': len(mass), 'triangles': len(data), 'stitches': len(initial), 'bends': len(bends), 'material_triangle_angle_range_degrees': angles, 'mass_max_difference_kg': mass_error,
                       'first_frame_above_strain_screen': next((r['frame'] for r in rows if r['material']['peak']['absolute_strain'] >= .15), None),
                       'first_frame_assembly_closed': next((s['global_frame'] for s in states if not s['diagnostics']['seam_assembly_remaining']), None),
                       'first_frame_seam_below_5mm': next((r['frame'] for r in rows if r['seam_gap_m'] < .005), None),
                       'comparison_max_absolute_errors': dict(maxima),
                       'checkpoints': [r for r in rows if r['frame'] in [0, 24, 27, 60, 120, 150, 151, 180]],
                       'job': {k: job[k] for k in ['started_at', 'completed_at', 'launcher_pid', 'child_pid', 'child_process_group', 'exit_code']}}
    control_meta = meta_by_case['control']
    original = copy.deepcopy(fixture['project'])
    original.setdefault('sewing_groups', [])
    for material in original['materials']:
        material['color'] = [struct.unpack('f', struct.pack('f', v))[0] for v in material['color']]
    assert original == control_meta['project']
    mesh_fields = ['rest_positions_m', 'uv_m', 'triangles', 'piece_ids', 'stitches', 'boundary', 'masses_kg']
    for name in ['zero-g-balance', 'no-avatar']:
        assert meta_by_case[name]['original_project'] == control_meta['project']
        expected = copy.deepcopy(control_meta['project'])
        if name == 'no-avatar':
            expected['mannequin']['capsules'] = []
        assert meta_by_case[name]['project'] == expected
        assert all(meta_by_case[name][key] == control_meta[key] for key in mesh_fields)
    prefix_keys = ['positions_m', 'velocities_m_per_s', 'settings_last_step']
    for left, right in zip(states_by_case['control'][:61], states_by_case['zero-g-balance'][:61]):
        assert all(left[key] == right[key] for key in prefix_keys)
        assert {k: v for k, v in left['diagnostics'].items() if k != 'step_seconds'} == {k: v for k, v in right['diagnostics'].items() if k != 'step_seconds'}
    mass = control_meta['masses_kg']
    last = all_rows['zero-g-balance']
    comparisons = {'zero_g_vs_control_t3_position_drift_m': drift(states_by_case['zero-g-balance'][180]['positions_m'], states_by_case['control'][180]['positions_m'], mass),
                   'zero_g_t1_to_t3_drift_m': drift(states_by_case['zero-g-balance'][180]['positions_m'], states_by_case['zero-g-balance'][60]['positions_m'], mass),
                   'zero_g_last_half_second_drift_m': drift(states_by_case['zero-g-balance'][180]['positions_m'], states_by_case['zero-g-balance'][150]['positions_m'], mass),
                   'zero_g_last_half_second_elastic_energy_change_J': last[180]['elastic_energy_J'] - last[150]['elastic_energy_J'],
                   'zero_g_last_half_second_peak_strain_frames_151_to_180_range': [min(r['material']['peak']['absolute_strain'] for r in last[151:]), max(r['material']['peak']['absolute_strain'] for r in last[151:])],
                   'no_avatar_vs_control_t1_position_drift_m': drift(states_by_case['no-avatar'][60]['positions_m'], states_by_case['control'][60]['positions_m'], mass)}
    points = {p['id']: p['position'] for p in original['points']}
    curves = {c['id']: c for c in original['curves']}
    def curve_length(identifier):
        curve = curves[identifier]
        assert curve['controls'] is None
        return length(sub(points[curve['end']], points[curve['start']]))
    seam_geometry = []
    for seam in original['seams']:
        matching = [s for s in control_meta['stitches'] if s['seam'] == seam['id']]
        offsets = [sub(stitch_point(s['a'], control_meta['rest_positions_m']), stitch_point(s['b'], control_meta['rest_positions_m'])) for s in matching]
        seam_geometry.append({'seam': seam['id'], 'source_correspondence': seam['correspondence'], 'side_lengths_m': [curve_length(seam['a']['curve']), curve_length(seam['b']['curve'])],
                              'max_arranged_xy_mismatch_m': max(math.hypot(*d[:2]) for d in offsets), 'arranged_z_gap_range_m': [min(d[2] for d in offsets), max(d[2] for d in offsets)]})
    loops = [{'name': name, 'source_perimeter_m': sum(curve_length(i) for i in ids), 'conditional_uniform_extension': 2 * math.pi * radius / sum(curve_length(i) for i in ids) - 1}
             for name, ids, radius in [('hem', [17, 46], .183), ('neck', [23, 24, 25, 52, 53, 54], .108), ('right_cuff', [20, 49], .068), ('left_cuff', [28, 57], .068)]]
    historical = {'status': 'not supplied; historical states not relabeled current'}
    if args.historical:
        old = read(args.historical / 'runs/rest-metadata.json')
        historical = {'source': 'e342471b18bc5d636d8e869ba0d374710ea15da3', 'array_equality': {k: control_meta[k] == old[k] for k in ['rest_positions_m', 'uv_m', 'triangles', 'piece_ids', 'masses_kg']}, 'same_node_drift': 'not valid when topology differs'}
    receipt = {'kind': 'independent_saved_output_review_no_simulator', 'source_commit': source_commit, 'source_verification': source_check,
               'binaries': binaries, 'input_sha256': inputs, 'self_checks': 'passed material inverse/force finite differences, UV mass, face/edge/corner distances and weighted drift',
               'fixture_normalization': 'Only deserialize missing sewing_groups as [] and color channels as f32, matching source types',
               'prefix': {'frames_0_through_60': 'exact positions, velocities, settings and all diagnostics except measured step_seconds', 'seam_targets': 'source recurrence reconstructed for every frame; nonnegative targets all zero at frame60', 'limits': 'No direct private-memory serialization; source multipliers are local to each substep and reset there'},
               'cases': cases, 'force_vector_max_absolute_errors_N': force_errors, 'comparisons': comparisons, 'authored_seam_geometry': seam_geometry, 'conditional_openings': loops, 'historical': historical,
               'limits': ['Private ignored evidence; not public delivery or reference parity.', 'No new simulation or production/default/fixture/threshold changes.', 'Static supported contact means analytic point-capsule/ground and excluded vertex-triangle distances only; no full edge-edge or continuous collision proof.', 'Conservative force gradients exclude contact/friction reactions and damping; sequential XPBD impulses need not equal endpoint gradients.', 'Low velocity, small frame changes and momentum changes do not prove force equilibrium or convergence.', 'Conditional perimeter estimates do not prove any material-axis strain lower bound; shear and migration remain possible.', 'No-avatar arithmetic result is never an original garment-quality pass.']}
    print(json.dumps(receipt, indent=2, allow_nan=False))


if __name__ == '__main__':
    main()
