"""Small independent static review; no product imports, builds or time integration."""
import hashlib
import itertools
import json
import math
import random
import subprocess
from decimal import Decimal, localcontext
from pathlib import Path

OUT = Path(__file__).resolve().parent
ROOT = OUT.parents[1]
COMMIT = '24dba7715233a7423088772d49c32a632d901a0c'
EPS = 2.220446049250313e-16


def add(a, b):
    return [x + y for x, y in zip(a, b)]


def sub(a, b):
    return [x - y for x, y in zip(a, b)]


def mul(a, s):
    return [x * s for x in a]


def dot(a, b):
    return sum(x * y for x, y in zip(a, b))


def cross(a, b):
    return [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]


def norm(a):
    return math.hypot(math.hypot(a[0], a[1]), a[2])


def fraction(p, a, b):
    axis = sub(b, a)
    squared = dot(axis, axis)
    return min(1., max(0., dot(sub(p, a), axis) / squared)) if squared > 0. else 0.


def transcribed_closest(tri, a, b):
    """Transparent f64 transcription; results are not an executable Rust check."""
    relative = [sub(tri[1], tri[0]), sub(tri[2], tri[0]), sub(a, tri[0]), sub(b, tri[0])]
    scale = max(abs(x) for p in relative for x in p)
    if not math.isfinite(scale):
        return 0., True, [1., 0., 0.], 0., scale
    if scale == 0.:
        return 0., False, [1., 0., 0.], 0., scale
    u, v, a, b = [[x / scale for x in p] for p in relative]
    tri = [[0., 0., 0.], u, v]
    axis = sub(b, a)
    best = [math.inf, [1., 0., 0.], 0.]
    def offer(bary, s):
        point = add(mul(u, bary[1]), mul(v, bary[2]))
        distance = norm(sub(point, add(a, mul(axis, s))))
        if distance < best[0]:
            best[:] = [distance, bary, s]
    for i, j in [(0, 1), (1, 2), (2, 0)]:
        p, q = tri[i], tri[j]
        edge = sub(q, p)
        def edge_offer(t, s):
            bary = [0., 0., 0.]
            bary[i], bary[j] = 1. - t, t
            offer(bary, s)
        edge_offer(0., fraction(p, a, b))
        edge_offer(1., fraction(q, a, b))
        edge_offer(fraction(a, p, q), 0.)
        edge_offer(fraction(b, p, q), 1.)
        n = cross(edge, axis)
        size = norm(n)
        if size > 0.:
            n = [x / size for x in n]
            delta = sub(p, a)
            t, s = dot(cross(axis, delta), n) / size, dot(cross(edge, delta), n) / size
            if 0. <= t <= 1. and 0. <= s <= 1.:
                edge_offer(t, s)
    normal = cross(u, v)
    size, squared = norm(normal), dot(normal, normal)
    longest = max(map(norm, [u, v, sub(v, u)]))
    longest_squared = longest * longest
    if (longest > 0. and longest_squared == 0.) or (size > 0. and squared == 0.):
        return 0., True, best[1], best[2], scale
    if size <= 1e-7 * longest_squared:
        rounding = 4. * EPS * norm([abs(u[1] * v[2]) + abs(u[2] * v[1]),
                                   abs(u[2] * v[0]) + abs(u[0] * v[2]),
                                   abs(u[0] * v[1]) + abs(u[1] * v[0])])
        altitude = (size + rounding) / longest if longest > 0. else 0.
        return max(0., best[0] - altitude) * scale, size > 0. or rounding > 0., best[1], best[2], scale
    def face_offer(p, s):
        bv, bw = dot(cross(p, v), normal) / squared, dot(cross(u, p), normal) / squared
        bary = [1. - bv - bw, bv, bw]
        if all(0. <= x <= 1. for x in bary):
            offer(bary, s)
    face_offer(a, 0.)
    face_offer(b, 1.)
    start, change = dot(a, normal), dot(axis, normal)
    if change != 0.:
        s = -start / change
        if 0. <= s <= 1.:
            face_offer(add(a, mul(axis, s)), s)
    return best[0] * scale, False, best[1], best[2], scale


def solve(matrix, rhs):
    n = len(rhs)
    rows = [r[:] + [v] for r, v in zip(matrix, rhs)]
    for i in range(n):
        pivot = max(range(i, n), key=lambda j: abs(rows[j][i]))
        if abs(rows[pivot][i]) < Decimal('1e-70'):
            return None
        rows[i], rows[pivot] = rows[pivot], rows[i]
        p = rows[i][i]
        rows[i] = [v / p for v in rows[i]]
        for j in range(n):
            if i != j:
                value = rows[j][i]
                rows[j] = [x - value * y for x, y in zip(rows[j], rows[i])]
    return [r[-1] for r in rows]


def independent_qp(tri, a, b):
    """Enumerate constrained-quadratic active sets at 100-digit precision."""
    with localcontext() as context:
        context.prec = 100
        dec = lambda p: [Decimal.from_float(float(x)) for x in p]
        tri, a, b = list(map(dec, tri)), dec(a), dec(b)
        u, v, axis, delta = sub(tri[1], tri[0]), sub(tri[2], tri[0]), sub(b, a), sub(tri[0], a)
        scale = max(abs(x) for p in [u, v, axis, delta] for x in p)
        if not scale:
            return 0.
        u, v, axis, delta = [[x / scale for x in p] for p in [u, v, axis, delta]]
        columns = [u, v, [-x for x in axis]]
        hessian = [[dot(x, y) for y in columns] for x in columns]
        linear = [dot(x, delta) for x in columns]
        constraints = [[-1, 0, 0], [0, -1, 0], [1, 1, 0], [0, 0, -1], [0, 0, 1]]
        bounds = [0, 0, 1, 0, 1]
        best = None
        for active_count in range(4):
            for active in itertools.combinations(range(5), active_count):
                matrix = [r[:] + [Decimal(constraints[j][i]) for j in active] for i, r in enumerate(hessian)]
                matrix += [[Decimal(x) for x in constraints[j]] + [Decimal(0)] * active_count for j in active]
                values = solve(matrix, [-x for x in linear] + [Decimal(bounds[j]) for j in active])
                if values is None:
                    continue
                z = values[:3]
                if any(dot(row, z) > bound + Decimal('1e-55') for row, bound in zip(constraints, bounds)):
                    continue
                point = [delta[k] + sum(columns[j][k] * z[j] for j in range(3)) for k in range(3)]
                squared = dot(point, point)
                best = squared if best is None else min(best, squared)
        assert best is not None
        return float(best.sqrt() * scale)


def self_check():
    tri = [[-1., -1., 0.], [1., -1., 0.], [0., 1., 0.]]
    for a, b, expected in [([0., 0., 1.], [0., 0., 2.], 1.),
                            ([0., 0., -1.], [0., 0., 1.], 0.),
                            ([-2., 0., 1.], [2., 0., 1.], 1.),
                            ([0., 2., 0.], [0., 3., 0.], 1.),
                            ([0., 0., 1.], [0., 0., 1.], 1.)]:
        assert abs(independent_qp(tri, a, b) - expected) < 1e-14
    # Independent static weighted-point correction, without any integration.
    beta, inverse_mass, depth = [.5, .5, 0.], [1., 3., 0.], .15
    denominator = sum(w * q * q for w, q in zip(inverse_mass, beta))
    normal_motion = [w * q * depth / denominator for w, q in zip(inverse_mass, beta)]
    assert normal_motion == [.075, .22499999999999998, 0.]
    assert abs(sum(q * move for q, move in zip(beta, normal_motion)) - depth) < 1e-14
    tangential_displacement, friction = .1, .5
    amount = min(friction * depth / denominator, tangential_displacement / denominator)
    friction_motion = [w * q * amount for w, q in zip(inverse_mass, beta)]
    assert abs(sum(q * move for q, move in zip(beta, friction_motion)) - .075) < 1e-14
    assert normal_motion[2] == friction_motion[2] == 0.


def main():
    self_check()
    source_paths = ['crates/weftforge-sim/src/' + name for name in ['capsule.rs', 'contact.rs', 'lib.rs', 'math.rs', 'tests.rs']]
    source_hashes = {p: hashlib.sha256(subprocess.check_output(['git', 'show', COMMIT + ':' + p], cwd=ROOT)).hexdigest() for p in source_paths}
    cases = []
    rng = random.Random(20261115)
    for i in range(48):
        tri = [[rng.uniform(-1., 1.) for k in range(3)] for j in range(3)]
        a, b = [[rng.uniform(-2., 2.) for k in range(3)] for j in range(2)]
        cases.append(('ordinary_' + str(i), tri, a, b))
    for exponent in [-5, -7, -9, -12]:
        height = 10. ** exponent
        for z in [0., 1e-6, .1]:
            tri = [[-1., 0., 0.], [1., 0., 0.], [0., height, 0.]]
            cases.append((f'skinny_{exponent}_{z}', tri, [0., height / 2, z], [0., height / 2, z]))
    for shift in [0., 1e-10, .1]:
        cases.append(('parallel_' + str(shift), [[-1., -1., 0.], [1., -1., 0.], [0., 1., 0.]], [-2., 0., shift], [2., 0., shift]))
        cases.append(('near_parallel_' + str(shift), [[-1., 0., shift], [1., 1e-11, shift], [0., -1., shift]], [-2., 1e-9, 0.], [2., 1.04e-9, 0.]))
    cases += [('line', [[-1., 0., 0.], [0., 0., 0.], [1., 0., 0.]], [0., 1., 0.], [0., 1., 0.]),
              ('point', [[0., 0., 0.]] * 3, [0., 1., 0.], [0., 2., 0.])]
    witness_path = ROOT / '.local/garment-fit-bound-review-20261114/static-witnesses.json'
    witnesses = json.loads(witness_path.read_text())
    for case in witnesses['cases']:
        if case['label'] not in ['original_grain180', 'Harbor_A180_gold', 'Harbor_B180_gold']:
            continue
        w = case['worst_edge_capsule_witness']
        cases.append((case['label'], w['triangle_positions_m'], w['capsule_axis_a_m'], w['capsule_axis_b_m']))
    for scale in [1e-10, 1e10]:
        for label, tri, a, b in cases[:6]:
            transform = lambda p: [scale * p[2], scale * p[0], scale * p[1]]
            cases.append((label + '_scaled_' + str(scale), list(map(transform, tri)), transform(a), transform(b)))
    assert len(cases) <= 128
    results, failures = [], []
    for label, tri, a, b in cases:
        distance, uncertain, bary, axis_t, scale = transcribed_closest(tri, a, b)
        truth = independent_qp(tri, a, b)
        tolerance = 5e-12 * max(scale, 1e-15)
        valid = all(0. <= x <= 1. for x in bary) and abs(sum(bary) - 1.) < 1e-13 and 0. <= axis_t <= 1.
        passed = valid and distance <= truth + tolerance and (uncertain or abs(distance - truth) <= tolerance)
        record = {'label': label, 'distance_transcribed_f64': distance, 'independent_qp_distance': truth,
                  'uncertain': uncertain, 'bary': bary, 'axis_fraction': axis_t, 'scale': scale,
                  'tolerance': tolerance, 'passed': passed}
        results.append(record)
        if not passed:
            failures.append(dict(record, triangle=tri, axis_a=a, axis_b=b))
    underflow = transcribed_closest([[-2e-170, -2e-170, 0.], [2e-170, -2e-170, 0.], [0., 2e-170, 0.]], [0., 0., -1.], [0., 0., 1.])
    assert underflow[0] == 0. and underflow[1]
    output = {'kind': 'bounded_independent_static_implementation_review', 'commit': COMMIT,
              'source_sha256': source_hashes, 'method_sha256': hashlib.sha256((OUT / 'METHOD.md').read_bytes()).hexdigest(),
              'script_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
              'case_count': len(cases), 'failures': failures, 'rows': results,
              'self_checks': 'passed', 'underflow_conservative_zero_check': 'passed',
              'limits': 'Python transcription is not executable Rust validation. No garment dynamics, Cargo, compile or full parity claim.'}
    (OUT / 'static-check.json').write_text(json.dumps(output, indent=2) + '\n')
    print(json.dumps({'cases': len(cases), 'failures': failures, 'uncertain_cases': sum(r['uncertain'] for r in results), 'self_checks': 'passed'}))


if __name__ == '__main__':
    main()
