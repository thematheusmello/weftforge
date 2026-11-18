"""Exact independent numeric functions extracted from the prior geometry review."""
import math


def sub(a, b):
    return [x - y for x, y in zip(a, b)]


def dot(a, b):
    return sum(x * y for x, y in zip(a, b))


def norm(a):
    return math.sqrt(dot(a, a))


def cross(a, b):
    return [a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]


def lerp(a, b, t):
    return [x + t * (y - x) for x, y in zip(a, b)]


def clamp(x):
    return min(1., max(0., x))


def closest_segments(p, q, a, b):
    """Minimize the two-parameter squared distance on a unit square."""
    u, v, w = sub(q, p), sub(b, a), sub(p, a)
    aa, bb, cc, dd, ee = dot(u, u), dot(u, v), dot(v, v), dot(u, w), dot(v, w)
    candidates = [(0., clamp(ee / cc) if cc else 0.),
                  (1., clamp((ee + bb) / cc) if cc else 0.),
                  (clamp(-dd / aa) if aa else 0., 0.),
                  (clamp((bb - dd) / aa) if aa else 0., 1.)]
    determinant = aa * cc - bb * bb
    if determinant > 1e-14 * aa * cc:
        s, t = (bb * ee - cc * dd) / determinant, (aa * ee - bb * dd) / determinant
        if 0. <= s <= 1. and 0. <= t <= 1.:
            candidates.append((s, t))
    s, t = min(candidates, key=lambda st: dot(sub(lerp(p, q, st[0]), lerp(a, b, st[1])),
                                            sub(lerp(p, q, st[0]), lerp(a, b, st[1]))))
    return norm(sub(lerp(p, q, s), lerp(a, b, t))), s, t
