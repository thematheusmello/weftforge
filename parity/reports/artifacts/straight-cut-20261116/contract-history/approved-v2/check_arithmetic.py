"""Small literal-oracle arithmetic check; never imports or invokes WeftForge."""
import json
from fractions import Fraction as F
from pathlib import Path

root = Path(__file__).resolve().parent
oracle = json.loads((root / 'ORACLE.json').read_text(), parse_float=F)
q = lambda x: F(str(x))

def area(vertices):
    # Translate first; this is arithmetic on literal polygons, not a cut algorithm.
    x0, y0 = vertices[0]
    v = [(q(x)-q(x0), q(y)-q(y0)) for x, y in vertices]
    return abs(sum(a*d-b*c for (a,b),(c,d) in zip(v, v[1:]+v[:1]))) / 2

def perimeter(vertices):
    # Every declared R/L segment here is axis-aligned.
    pairs = zip(vertices, vertices[1:]+vertices[:1])
    return sum(abs(q(a)-q(c))+abs(q(b)-q(d)) for (a,b),(c,d) in pairs)

for key in ('R240', 'L'):
    case = oracle[key]
    fixture = json.loads((root / case['input']).read_text(), parse_float=F)['project']
    cut = case['cut'] if key == 'R240' else case['valid']
    original_vertices = [p['position'] for p in fixture['points']]
    assert area(original_vertices) == case['parent']['area']
    assert perimeter(original_vertices) == case['parent']['perimeter']
    for child in ('retained', 'new'):
        c = cut[child]
        assert area(c['polygon']) == c['area']
        assert perimeter(c['polygon']) == c['perimeter']
        assert c['area'] * oracle['default_density'] == c['mass']
    assert cut['retained']['area'] + cut['new']['area'] == cut['union_area'] == case['parent']['area']
    assert cut['retained']['mass'] + cut['new']['mass'] == cut['mass_sum'] == case['parent']['mass']
    assert cut['retained']['perimeter'] + cut['new']['perimeter'] == cut['perimeter_sum']
    assert case['parent']['perimeter'] + 2*cut['cut_length'] == cut['perimeter_sum']
    n = fixture['next_id']
    assert cut['new']['id'] == n+4 and cut['next_id'] == n+9
    points = {p['id']: p['position'] for p in fixture['points']}
    points.update({int(i): xy for i,xy in cut['new_points'].items()})
    for dest, source in cut['copy_bits']:
        assert points[dest] == points[source]
    for child in ('retained', 'new'):
        edges = [cut['all_curve_incidences'][str(i)] for i in cut[child]['boundary']]
        assert all(b == c for (_,b),(c,_) in zip(edges, edges[1:]+edges[:1]))
        assert [points[a] for a,b in edges] == cut[child]['polygon']
    assert len(set(cut['retained']['boundary']) & set(cut['new']['boundary'])) == 0
assert oracle['R240']['reopen_edit']['new_area'] * oracle['default_density'] == oracle['R240']['reopen_edit']['new_mass']
assert oracle['R240']['cut']['retained']['area'] + oracle['R240']['reopen_edit']['new_area'] == oracle['R240']['reopen_edit']['total_area']
assert oracle['R240']['reopen_edit']['total_area'] * oracle['default_density'] == oracle['R240']['reopen_edit']['total_mass']
assert oracle['R240']['history'][-1]['new_piece'] == 20+4
assert oracle['R240']['history'][-1]['next_id'] == 20+9
assert oracle['annotations']['contained']['after_cut']['new_piece'] == 17+4
assert oracle['annotations']['contained']['after_cut']['next_id'] == 17+9
# Sum the published CUT-R1 table; CUT-R2 is its stated macro sum.
assert sum([12,25,2,2,15,2,4,18,8,6,2,22,2,5,4,7,7,14,1]) == 158
assert sum([12,2,5,2,15,2,4,6,1]) == 49
print(json.dumps({'status':'pass','scope':'literal rational areas/perimeters/masses/IDs/walks and static action sums only','R1_static':158,'R2_static':49,'production_reference_GUI_Cargo_evaluator_Git_execution':False}, indent=2))
