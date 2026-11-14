from pathlib import Path
import sys,json,math,collections,hashlib
fixture=Path(sys.argv[1]); receipt=Path(sys.argv[2]); output=Path(sys.argv[3])
p=json.loads(fixture.read_text())['project']; m=json.loads(receipt.read_text())['mesh']
uv=m['uv']; ids=m['piece_ids']; triangles=m['triangles']; samples=m['boundary']
points={v['id']:v['position'] for v in p['points']}; curves={v['id']:v for v in p['curves']}
owner={c:v['id'] for v in p['pieces'] for c in v['boundary']}
def cross(a,b,c):return (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])
def evaluate(c,t):
    a,b=points[c['start']],points[c['end']]
    if c['controls'] is None:return [a[k]*(1-t)+b[k]*t for k in range(2)]
    u,v=c['controls']; s=1-t
    return [s**3*a[k]+3*s*s*t*u[k]+3*s*t*t*v[k]+t**3*b[k] for k in range(2)]
def area(c):
    a,b=points[c['start']],points[c['end']]
    if c['controls'] is None:return (a[0]*b[1]-a[1]*b[0])/2
    u,v=c['controls']
    z=[[a[k],3*(u[k]-a[k]),3*(a[k]-2*u[k]+v[k]),-a[k]+3*u[k]-3*v[k]+b[k]] for k in range(2)]
    return sum((z[0][i]*j*z[1][j]-z[1][i]*j*z[0][j])/(i+j) for i in range(4) for j in range(1,4))/2
assert len(uv)==len(ids)==len(m['positions'])<=100000
assert len(triangles)<=200000
assert m['doc_revision']==p['revision']
assert len({tuple(sorted(t)) for t in triangles})==len(triangles)
edges=collections.Counter(); directed=collections.Counter(); areas=collections.defaultdict(float); faces=collections.Counter(); used=set(); max_edge=0.; min_area=math.inf
parent=list(range(len(uv)))
def root(a):
    while a!=parent[a]:parent[a]=parent[parent[a]];a=parent[a]
    return a
for t in triangles:
    assert len(set(t))==3 and all(0<=i<len(uv) for i in t)
    assert len({ids[i] for i in t})==1
    a,b,c=(uv[i] for i in t); x=cross(a,b,c)/2
    assert x>5e-13; min_area=min(min_area,x); areas[ids[t[0]]]+=x;faces[ids[t[0]]]+=1;used.update(t)
    for a,b in zip(t,t[1:]+t[:1]):
        edges[tuple(sorted((a,b)))]+=1;directed[a,b]+=1
        L=math.dist(uv[a],uv[b]);max_edge=max(max_edge,L)
        assert L<=1.4*p['resolution']+1e-12,(L,1.4*p['resolution'])
        parent[root(a)]=root(b)
assert len(used)==len(uv)
assert all(v in (1,2) for v in edges.values())
assert all(n==1 or directed[a,b]==directed[b,a]==1 for (a,b),n in edges.items())
bycurve=collections.defaultdict(list);boundary_expected=set();boundary_area=collections.defaultdict(float); boundary_error=0.
for s in samples:
    assert owner[s['curve']]==s['piece']==ids[s['vertex']]
    assert math.isfinite(s['t']) and 0<=s['t']<=1
    error=math.dist(uv[s['vertex']],evaluate(curves[s['curve']],s['t']))
    assert error<2e-12;boundary_error=max(boundary_error,error)
    bycurve[s['curve']].append(s)
assert set(bycurve)==set(curves)
for curve,ss in bycurve.items():
    ss.sort(key=lambda s:s['t']);assert ss[0]['t']==0 and ss[-1]['t']==1
    for a,b in zip(ss,ss[1:]):
        assert a['t']<b['t']
        edge=tuple(sorted((a['vertex'],b['vertex'])));assert edge not in boundary_expected;boundary_expected.add(edge)
        x,y=uv[a['vertex']],uv[b['vertex']]
        boundary_area[a['piece']]+=(x[0]*y[1]-x[1]*y[0])/2
assert {e for e,n in edges.items() if n==1}==boundary_expected
piece_result=[]
for piece in p['pieces']:
    i=piece['id'];vs=[v for v,j in enumerate(ids) if j==i];es=sum(1 for a,b in edges if ids[a]==i);fs=faces[i]
    assert len(vs)-es+fs==1
    assert len({root(v) for v in vs})==1
    assert abs(areas[i]-boundary_area[i])<1e-9
    analytic=abs(sum(area(curves[c]) for c in piece['boundary']))
    perimeter=sum(math.dist(uv[a],uv[b]) for a,b in boundary_expected if ids[a]==i)
    assert abs(areas[i]-analytic)<p['resolution']*.01*perimeter
    piece_result.append({'piece':i,'vertices':len(vs),'edges':es,'triangles':fs,'mesh_area_m2':areas[i],'boundary_polygon_area_m2':boundary_area[i],'analytic_area_m2':analytic,'analytic_error_m2':areas[i]-analytic,'euler_characteristic':len(vs)-es+fs,'connected_components':1})
for pos,u in zip(m['positions'],uv):assert math.dist(pos,[u[0],u[1],0])<1e-12
assert len(m['stitches'])>0
for stitch in m['stitches']:
    group=next(g for g in p['sewing_groups'] if g['id']==stitch['seam'])
    for side in ('a','b'):
        w=stitch[side]; assert 0<=w['weight']<=1
        owners={owner[v['curve']] for v in group[side]}
        assert len({ids[v] for v in w['vertices']})==1 and ids[w['vertices'][0]] in owners
        matches=[]
        for interval in group[side]:
            ts=[next((s['t'] for s in bycurve[interval['curve']] if s['vertex']==v),None) for v in w['vertices']]
            if all(t is not None for t in ts):
                t=(1-w['weight'])*ts[0]+w['weight']*ts[1]
                matches.append(min(interval['t0'],interval['t1'])-1e-12<=t<=max(interval['t0'],interval['t1'])+1e-12)
        assert any(matches)
result={'fixture_sha256':hashlib.sha256(fixture.read_bytes()).hexdigest(),'receipt_sha256':hashlib.sha256(receipt.read_bytes()).hexdigest(),'vertices':len(uv),'triangles':len(triangles),'boundary_records':len(samples),'boundary_edges':len(boundary_expected),'stitches':len(m['stitches']),'max_edge_m':max_edge,'max_edge_limit_m':1.4*p['resolution'],'min_triangle_area_m2':min_area,'boundary_parameter_position_error_m':boundary_error,'pieces':piece_result,'checks':'resource limits, positive unique faces, all vertices used, piece ownership, oriented manifold edges, connected disk topology, max-edge threshold, exact boundary edges and curve parameters, area conservation against boundary, analytic area tolerance, placement, stitch side ownership'}
output.write_text(json.dumps(result,indent=2)+'\n'); print(json.dumps(result,indent=2))
