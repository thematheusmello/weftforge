"""Exact prior independent100digit active-set oracle; no solver transcription."""
import itertools
import math
from decimal import Decimal, localcontext


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
