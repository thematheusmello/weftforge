"""Independent authored-geometry math; no WeftForge imports or executables."""
from decimal import Decimal, localcontext
from functools import lru_cache
import math

D = Decimal
ZERO, ONE = D(0), D(1)
ABS_BUDGET = D("1e-8")


def dec(value):
    return D.from_float(value) if isinstance(value, float) else D(value)


@lru_cache(None)
def gauss_nodes(n):
    with localcontext() as ctx:
        ctx.prec = 70
        answer = []
        for k in range(1, n + 1):
            x = D(str(math.cos(math.pi * (k - 0.25) / (n + 0.5))))
            for _ in range(30):
                p0, p1 = ONE, x
                for j in range(2, n + 1):
                    p0, p1 = p1, ((2 * j - 1) * x * p1 - (j - 1) * p0) / j
                derivative = n * (x * p1 - p0) / (x * x - ONE)
                step = p1 / derivative
                x -= step
                if abs(step) < D("1e-65"):
                    break
            else:
                raise ArithmeticError("independent Legendre root did not converge")
            weight = 2 / ((ONE - x * x) * derivative * derivative)
            answer.append((+x, +weight))
        return tuple(answer)


def gauss_integral(function, a, b, n):
    middle, half = (a + b) / 2, (b - a) / 2
    return half * sum((weight * function(middle + half * x)
                       for x, weight in gauss_nodes(n)), ZERO)


def integrate(function, a, b, tolerance=D("1e-25"), depth=0):
    low = gauss_integral(function, a, b, 12)
    high = gauss_integral(function, a, b, 24)
    if abs(high - low) <= tolerance:
        return high
    if depth >= 18:
        raise ArithmeticError("independent quadrature work limit")
    middle = (a + b) / 2
    return (integrate(function, a, middle, tolerance / 2, depth + 1)
            + integrate(function, middle, b, tolerance / 2, depth + 1))


def simpson_check(function, a, b, tolerance=D("1e-17")):
    """Different quadrature family verifies curved totals before freeze."""
    def visit(a, b, fa, fm, fb, previous, tolerance, depth):
        middle = (a + b) / 2
        fl, fr = function((a + middle) / 2), function((middle + b) / 2)
        left = (middle - a) * (fa + 4 * fl + fm) / 6
        right = (b - middle) * (fm + 4 * fr + fb) / 6
        delta = left + right - previous
        if abs(delta) <= 15 * tolerance:
            return left + right + delta / 15
        if depth >= 24:
            raise ArithmeticError("independent Simpson work limit")
        return (visit(a, middle, fa, fl, fm, left, tolerance / 2, depth + 1)
                + visit(middle, b, fm, fr, fb, right, tolerance / 2, depth + 1))
    middle = (a + b) / 2
    fa, fm, fb = function(a), function(middle), function(b)
    return visit(a, b, fa, fm, fb, (b - a) * (fa + 4 * fm + fb) / 6,
                 tolerance, 0)


class Geometry:
    def __init__(self, document):
        self.project = document["project"]
        self.points = {p["id"]: tuple(map(dec, p["position"]))
                       for p in self.project["points"]}
        self.curves = {c["id"]: c for c in self.project["curves"]}
        self.cache = {}

    def controls(self, curve):
        c = self.curves[curve]
        p0, p3 = self.points[c["start"]], self.points[c["end"]]
        if c["controls"] is None:
            return p0, p3
        return p0, tuple(map(dec, c["controls"][0])), tuple(map(dec, c["controls"][1])), p3

    def point(self, curve, t):
        t = dec(t)
        rows = self.controls(curve)
        while len(rows) > 1:
            rows = tuple(tuple((ONE - t) * a + t * b for a, b in zip(p, q))
                         for p, q in zip(rows, rows[1:]))
        return rows[0]

    def speed(self, curve, t):
        controls = self.controls(curve)
        degree = len(controls) - 1
        rows = tuple(tuple(degree * (b - a) for a, b in zip(p, q))
                     for p, q in zip(controls, controls[1:]))
        while len(rows) > 1:
            rows = tuple(tuple((ONE - t) * a + t * b for a, b in zip(p, q))
                         for p, q in zip(rows, rows[1:]))
        return sum((x * x for x in rows[0]), ZERO).sqrt()

    def length(self, curve, a, b):
        a, b = sorted((dec(a), dec(b)))
        key = curve, a, b
        if key not in self.cache:
            if len(self.controls(curve)) == 2:
                self.cache[key] = self.speed(curve, ZERO) * (b - a)
            else:
                self.cache[key] = integrate(lambda t: self.speed(curve, t), a, b)
        return self.cache[key]

    def parameter(self, interval, distance):
        a, b = map(dec, (interval["t0"], interval["t1"]))
        total = self.length(interval["curve"], a, b)
        if distance <= ZERO:
            return a
        if distance >= total:
            return b
        if len(self.controls(interval["curve"])) == 2:
            return a + (b - a) * distance / total
        low, high = ZERO, ONE
        for _ in range(76):
            middle = (low + high) / 2
            t = a + (b - a) * middle
            if self.length(interval["curve"], a, t) < distance:
                low = middle
            else:
                high = middle
        return a + (b - a) * (low + high) / 2


def interpolate(mapping, value, inverse=False):
    pairs = [(dec(pair[1]), dec(pair[0])) if inverse else tuple(map(dec, pair))
             for pair in mapping]
    if value <= ZERO:
        return ZERO
    if value >= ONE:
        return ONE
    for (x0, y0), (x1, y1) in zip(pairs, pairs[1:]):
        if x0 <= value <= x1:
            return y0 + (y1 - y0) * (value - x0) / (x1 - x0)
    raise ValueError("value outside map")


class Path:
    def __init__(self, geometry, intervals):
        self.geometry, self.intervals = geometry, intervals
        self.lengths = [geometry.length(i["curve"], i["t0"], i["t1"])
                        for i in intervals]
        self.total = sum(self.lengths, ZERO)
        self.knots = [ZERO]
        for length in self.lengths:
            self.knots.append(self.knots[-1] + length / self.total)
        self.knots[-1] = ONE

    def resolve(self, fraction, owner):
        interval = self.intervals[owner]
        distance = (fraction - self.knots[owner]) * self.total
        if fraction == self.knots[owner]:
            t = dec(interval["t0"])
        elif fraction == self.knots[owner + 1]:
            t = dec(interval["t1"])
        else:
            t = self.geometry.parameter(interval, distance)
        return t, self.geometry.point(interval["curve"], t)

    def owner(self, fraction):
        for index, (low, high) in enumerate(zip(self.knots, self.knots[1:])):
            if low < fraction < high:
                return index
        raise ValueError("owner requires an open-cell fraction")

    def fraction(self, owner, t):
        interval = self.intervals[owner]
        return self.knots[owner] + self.geometry.length(interval["curve"], interval["t0"], t) / self.total


def group_expectation(geometry, group):
    a, b = Path(geometry, group["a"]), Path(geometry, group["b"])
    knots = set(a.knots)
    knots.update(interpolate(group["correspondence"], v, inverse=True) for v in b.knots)
    knots.update(dec(pair[0]) for pair in group["correspondence"])
    knots = sorted(knots)
    portions, details = [], []
    for u0, u1 in zip(knots, knots[1:]):
        middle = (u0 + u1) / 2
        ai = a.owner(middle)
        bi = b.owner(interpolate(group["correspondence"], middle))
        v0, v1 = (interpolate(group["correspondence"], u) for u in (u0, u1))
        ta0, pa0 = a.resolve(u0, ai)
        ta1, pa1 = a.resolve(u1, ai)
        tb0, pb0 = b.resolve(v0, bi)
        tb1, pb1 = b.resolve(v1, bi)
        portions.append({"a": {"curve": group["a"][ai]["curve"], "t0": float(ta0), "t1": float(ta1)},
                         "b": {"curve": group["b"][bi]["curve"], "t0": float(tb0), "t1": float(tb1)}})
        details.append({"u": [str(u0), str(u1)], "v": [str(v0), str(v1)],
                        "owners": [ai, bi], "a_t": [str(ta0), str(ta1)], "b_t": [str(tb0), str(tb1)],
                        "a_points": [[str(x) for x in p] for p in (pa0, pa1)],
                        "b_points": [[str(x) for x in p] for p in (pb0, pb1)]})
    return {"id": group["id"], "lengths": [float(a.total), float(b.total)],
            "portions": portions, "independent_details": details}


def independent_checks(document):
    geometry = Geometry(document)
    results = []
    for curve, c in geometry.curves.items():
        if c["controls"] is None:
            continue
        primary = geometry.length(curve, ZERO, ONE)
        second = simpson_check(lambda t: geometry.speed(curve, t), ZERO, ONE)
        delta = abs(primary - second)
        if delta >= D("1e-15"):
            raise ArithmeticError("independent quadrature families disagree")
        controls = geometry.controls(curve)
        if all(p[1] == controls[0][1] for p in controls):
            analytic = abs(controls[-1][0] - controls[0][0])
            if abs(primary - analytic) >= D("1e-24"):
                raise ArithmeticError("collinear analytic length mismatch")
        results.append({"curve": curve, "gauss_length": str(primary),
                        "simpson_length": str(second), "difference": str(delta)})
    return results
