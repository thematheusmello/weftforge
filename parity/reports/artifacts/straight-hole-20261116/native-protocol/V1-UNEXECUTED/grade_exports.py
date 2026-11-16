"""Independent literal R240 SVG/OBJ assessment; stdlib only, no product imports."""
import math
import re
import xml.etree.ElementTree as ET

OUTER = [(0., 0.), (.24, 0.), (.24, .36), (0., .36)]
HOLE = [(.08, .12), (.08, .24), (.16, .24), (.16, .12)]
P_TOL, A_TOL = 1e-12, 1e-10


def require(ok, message):
    if not ok:
        raise AssertionError(message)


def area(polygon):
    return math.fsum(a[0] * b[1] - b[0] * a[1] for a, b in zip(polygon, polygon[1:] + polygon[:1])) / 2


def cross(a, b, p):
    return (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0])


def clip(subject, boundary):
    for a, b in zip(boundary, boundary[1:] + boundary[:1]):
        old, subject = subject, []
        if not old:
            break
        p, dp = old[-1], cross(a, b, old[-1])
        for q in old:
            dq = cross(a, b, q)
            if (dp >= 0) != (dq >= 0):
                t = dp / (dp - dq)
                subject.append(tuple(p[k] + t * (q[k] - p[k]) for k in (0, 1)))
            if dq >= 0:
                subject.append(q)
            p, dp = q, dq
    return subject


def close(actual, expected, tolerance=P_TOL):
    require(len(actual) == len(expected) and all(math.isfinite(a) and abs(a - e) <= tolerance
            for a, e in zip(actual, expected)), 'export coordinates/scale differ')


def grade_svg(raw):
    require(len(raw) <= 16 * 1024 * 1024 and b'<!DOCTYPE' not in raw and b'<!ENTITY' not in raw, 'unsupported SVG declaration/size')
    tree = ET.fromstring(raw.decode('utf-8'))
    tag = lambda node: node.tag.rsplit('}', 1)[-1]
    require(tree.tag == '{http://www.w3.org/2000/svg}svg' and set(tree.attrib) == {'width', 'height', 'viewBox'} and
            all(node.tag in ('{http://www.w3.org/2000/svg}' + name for name in ('title', 'desc', 'path')) for node in tree), 'unsupported SVG root/primitive')
    require(tree.get('width', '').endswith('mm') and tree.get('height', '').endswith('mm'), 'SVG needs physical mm units')
    close([float(tree.get(k)[:-2]) for k in ('width', 'height')], [240., 360.], 1e-9)
    close([float(x) for x in tree.get('viewBox').split()], [0., -360., 240., 360.], 1e-9)
    paths = [node for node in tree if tag(node) == 'path']
    require(len(paths) == 1 and not list(paths[0]), 'SVG must contain one compound piece path')
    path = paths[0]
    require(set(path.attrib) == {'id', 'data-name', 'd', 'fill', 'fill-rule', 'stroke', 'stroke-width'} and
            path.get('id') == 'piece-2' and path.get('data-name') == 'Original Hole R240' and
            path.get('fill-rule') == 'evenodd', 'SVG piece identity/evenodd differs')
    pattern = r'[A-Za-z]|[-+]?(?:\d*\.\d+|\d+\.?\d*)(?:[eE][-+]?\d+)?'
    data = path.get('d', '')
    require(not re.sub(pattern, '', data).strip(' \t\r\n,'), 'unsupported SVG path token')
    tokens, loops, current, index = re.findall(pattern, data), [], [], 0
    while index < len(tokens):
        command = tokens[index]
        index += 1
        if command == 'Z':
            require(len(current) >= 4, 'SVG contour is not closed')
            if current[-1] == current[0]:
                current.pop()
            loops.append(current)
            current = []
        else:
            require(command in ('M', 'L') and (command == 'M') == (not current), 'SVG must use closed M/L/Z contours')
            current.append((float(tokens[index]) / 1000., -float(tokens[index + 1]) / 1000.))
            index += 2
    require(not current and len(loops) == 2, 'SVG needs exactly exterior+hole subpaths')
    for actual, expected in zip(loops, (OUTER, HOLE)):
        close(sum((list(p) for p in actual), []), sum((list(p) for p in expected), []))
    return {'compound_paths': 1, 'closed_subpaths': 2, 'physical_scale': 'mm', 'fill_rule': 'evenodd'}


def grade_obj(raw):
    require(len(raw) <= 16 * 1024 * 1024, 'OBJ exceeds native artifact bound')
    vertices, uv, normals, faces, groups = [], [], [], [], []
    group = None
    for line in raw.decode('utf-8').splitlines():
        parts = line.split()
        if not parts or parts[0].startswith('#'):
            continue
        kind, values = parts[0], parts[1:]
        if kind in ('v', 'vt', 'vn'):
            vector = tuple(float(x) for x in values)
            require(len(vector) == (2 if kind == 'vt' else 3) and all(math.isfinite(x) for x in vector), 'invalid OBJ vector')
            {'v': vertices, 'vt': uv, 'vn': normals}[kind].append(vector)
        elif kind == 'g':
            require(values == ['piece_2'], 'OBJ has another piece/island group')
            group = values[0]
            groups.append(group)
        elif kind == 'f':
            require(group == 'piece_2' and len(values) == 3, 'OBJ needs triangular piece2 faces')
            face = [tuple(int(x) - 1 for x in token.split('/')) for token in values]
            require(all(len(index) == 3 and min(index) >= 0 for index in face), 'OBJ needs positive v/vt/vn indices')
            faces.append(face)
        else:
            raise AssertionError('unsupported/non-surface OBJ primitive')
    require(3 <= len(vertices) <= 100000 and 0 < len(faces) <= 200000 and groups, 'OBJ is empty/over cap')
    edges, triangles, used = {}, [], set()
    for number, face in enumerate(faces):
        polygon, ids = [], []
        for vi, ti, ni in face:
            require(vi < len(vertices) and ti < len(uv) and ni < len(normals), 'OBJ index outside table')
            close(vertices[vi], [*uv[ti], 0.])
            close(normals[ni], [0., 0., 1.])
            require(-P_TOL <= uv[ti][0] <= .24 + P_TOL and -P_TOL <= uv[ti][1] <= .36 + P_TOL, 'OBJ outside exterior')
            polygon.append(uv[ti]); ids.append(vi); used.add(vi)
        require(len(set(ids)) == 3 and area(polygon) > 0, 'OBJ face is degenerate/non-CCW')
        require(abs(area(clip(polygon, list(reversed(HOLE))))) <= 1e-14, 'OBJ face interior fills the hole')
        triangles.append(polygon)
        for a, b in zip(ids, ids[1:] + ids[:1]):
            edges.setdefault(tuple(sorted((a, b))), []).append((a, b, number))
    require(used == set(range(len(vertices))) and abs(math.fsum(map(area, triangles)) - .0768) <= A_TOL, 'OBJ surface area/used vertices differ')
    adjacency = {n: set() for n in range(len(faces))}
    boundary = []
    for occurrences in edges.values():
        require(len(occurrences) in (1, 2), 'OBJ nonmanifold edge')
        if len(occurrences) == 1:
            boundary.append(occurrences[0][:2])
        else:
            a, b = occurrences
            require(a[:2] == (b[1], b[0]), 'OBJ internal edge orientations disagree')
            adjacency[a[2]].add(b[2]); adjacency[b[2]].add(a[2])
    reached, pending = set(), [0]
    while pending:
        face = pending.pop()
        if face not in reached:
            reached.add(face); pending.extend(adjacency[face] - reached)
    require(len(reached) == len(faces) and len(vertices) - len(edges) + len(faces) == 0, 'OBJ is not one annulus')
    graph = {}
    for a, b in boundary:
        graph.setdefault(a, set()).add(b); graph.setdefault(b, set()).add(a)
    require(all(len(neighbors) == 2 for neighbors in graph.values()), 'OBJ boundary is not closed loops')
    components = []
    while graph:
        component, pending = set(), [next(iter(graph))]
        while pending:
            vertex = pending.pop()
            if vertex not in component:
                component.add(vertex); pending.extend(graph.get(vertex, set()) - component)
        components.append(component)
        for vertex in component:
            graph.pop(vertex)
    require(len(components) == 2, 'OBJ needs exactly two boundary loops')
    perimeters = []
    for component in components:
        loop_edges = [(vertices[a][:2], vertices[b][:2]) for a, b in boundary if a in component]
        matches = []
        for polygon in (OUTER, HOLE):
            def on(p, a, b):
                return abs(cross(a, b, p)) <= P_TOL * math.dist(a, b) and all(min(a[k], b[k]) - P_TOL <= p[k] <= max(a[k], b[k]) + P_TOL for k in (0, 1))
            matches.append(all(any(on(p, a, b) and on(q, a, b) for a, b in zip(polygon, polygon[1:] + polygon[:1])) for p, q in loop_edges))
        require(sum(matches) == 1, 'OBJ boundary does not follow complete authored loops')
        perimeters.append(math.fsum(math.dist(a, b) for a, b in loop_edges))
    close(sorted(perimeters), [.4, 1.2])
    # shortcut: two million pair checks bound this R240 lab assessment; use a separately bounded spatial check for denser cases.
    checks = 0
    order = sorted(range(len(triangles)), key=lambda n: min(p[0] for p in triangles[n]))
    for offset, left in enumerate(order):
        a = triangles[left]
        for right in order[offset + 1:]:
            b = triangles[right]
            if min(p[0] for p in b) > max(p[0] for p in a):
                break
            checks += 1
            require(checks <= 2000000, 'lab triangle-pair assessment budget exceeded; unverified')
            require(abs(area(clip(a, b))) <= 1e-14, 'OBJ face interiors overlap')
    return {'surface_area_m2': math.fsum(map(area, triangles)), 'piece_components': 1,
            'boundary_components': 2, 'euler_characteristic': 0, 'units_and_UV': 'metres; [u,v,0]; Y up',
            'actual_Cloth_masses_verified': False}
