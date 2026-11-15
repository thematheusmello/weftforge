"""Exact independent closest-feature routines; never imports the product."""
import geometry_math as g


def barycentric(point, triangle):
    a, b, c = triangle
    u, v, w = g.sub(b, a), g.sub(c, a), g.sub(point, a)
    aa, ab, bb = g.dot(u, u), g.dot(u, v), g.dot(v, v)
    determinant = aa * bb - ab * ab
    if determinant <= 1e-14 * aa * bb:
        return None
    x = (bb * g.dot(w, u) - ab * g.dot(w, v)) / determinant
    y = (aa * g.dot(w, v) - ab * g.dot(w, u)) / determinant
    return [1. - x - y, x, y]


def interpolate(triangle, bary):
    return [sum(w * p[k] for w, p in zip(bary, triangle)) for k in range(3)]


def point_triangle(point, triangle):
    candidates = []
    for i in range(3):
        distance, _, t = g.closest_segments(point, point, triangle[i], triangle[(i + 1) % 3])
        bary = [0., 0., 0.]
        bary[i], bary[(i + 1) % 3] = 1. - t, t
        candidates.append((distance, bary, 'edge_or_vertex'))
    bary = barycentric(point, triangle)
    if bary is not None and min(bary) >= 0.:
        projected = interpolate(triangle, bary)
        candidates.append((g.norm(g.sub(point, projected)), bary, 'face'))
    return min(candidates)


def triangle_axis(triangle, capsule):
    """Complete static closest-feature candidates for an axis segment and triangle."""
    a, b = capsule['a'], capsule['b']
    candidates = []
    for t, point in [(0., a), (1., b)]:
        distance, bary, feature = point_triangle(point, triangle)
        candidates.append((distance, bary, t, 'axis_endpoint_' + feature))
    for i in range(3):
        distance, t_edge, t_axis = g.closest_segments(triangle[i], triangle[(i + 1) % 3], a, b)
        bary = [0., 0., 0.]
        bary[i], bary[(i + 1) % 3] = 1. - t_edge, t_edge
        candidates.append((distance, bary, t_axis, 'triangle_edge_axis'))
    normal = g.cross(g.sub(triangle[1], triangle[0]), g.sub(triangle[2], triangle[0]))
    denominator = g.dot(normal, g.sub(b, a))
    if denominator != 0.:
        t = g.dot(normal, g.sub(triangle[0], a)) / denominator
        if 0. <= t <= 1.:
            point = g.lerp(a, b, t)
            bary = barycentric(point, triangle)
            if bary is not None and min(bary) >= 0.:
                candidates.append((g.norm(g.sub(interpolate(triangle, bary), point)), bary, t,
                                   'axis_intersects_face'))
    return min(candidates)
