#!/usr/bin/env python3
"""Independent geometry measurements for original straight-boundary garment fixtures."""
import json
import math
import pathlib
import re
import xml.etree.ElementTree as ET
import sys

def sub(a,b):return [x-y for x,y in zip(a,b)]
def norm(v):return math.sqrt(sum(x*x for x in v))
def lerp(a,b,t):return [x+(y-x)*t for x,y in zip(a,b)]
def strain(world,uv):
    a,b=sub(uv[1],uv[0]),sub(uv[2],uv[0]);det=a[0]*b[1]-b[0]*a[1]
    assert abs(det)>1e-12,'degenerate material triangle'
    e,f=sub(world[1],world[0]),sub(world[2],world[0])
    u=[(e[i]*b[1]-f[i]*a[1])/det for i in range(3)]
    v=[(f[i]*a[0]-e[i]*b[0])/det for i in range(3)]
    return max(abs(norm(u)-1),abs(norm(v)-1))

def measure(project_path,obj_path,svg_path=None):
    project=json.loads(pathlib.Path(project_path).read_text())['project']
    vertices=[];uv=[];normals=[];faces=[];group=None
    for line in pathlib.Path(obj_path).read_text().splitlines():
        fields=line.split()
        if not fields:continue
        if fields[0]=='v':vertices.append(list(map(float,fields[1:])))
        elif fields[0]=='vt':uv.append(list(map(float,fields[1:])))
        elif fields[0]=='vn':normals.append(list(map(float,fields[1:])))
        elif fields[0]=='g':group=int(fields[1].removeprefix('piece_'))
        elif fields[0]=='f':faces.append((group,[tuple(int(i)-1 for i in f.split('/')) for f in fields[1:]]))
    assert vertices and faces and normals and uv,'required mesh data absent'
    assert all(len(v)==3 and all(math.isfinite(x) for x in v) for v in vertices)
    assert all(len(n)==3 and abs(norm(n)-1)<1e-5 for n in normals),'unit normals required'
    pieces={p['id']:p for p in project['pieces']};materials={m['id']:m for m in project['materials']}
    by_piece={pid:[] for pid in pieces};max_strain=0.;max_contact=0.;used=set()
    for pid,face in faces:
        assert pid in pieces and len(face)==3,'invalid face/piece group'
        assert all(0<=a<len(vertices) and 0<=b<len(uv) and 0<=c<len(normals) for a,b,c in face),'face index out of range'
        world=[vertices[v] for v,_,_ in face];rest=[uv[u] for _,u,_ in face]
        max_strain=max(max_strain,strain(world,rest));by_piece[pid].append((world,rest))
        thickness=materials[pieces[pid]['material']]['collision_thickness']
        for vertex,_,_ in face:
            if (pid,vertex) in used:continue
            used.add((pid,vertex));p=vertices[vertex]
            max_contact=max(max_contact,thickness-p[1])
            for capsule in project['mannequin']['capsules']:
                a,b=capsule['a'],capsule['b'];ab=sub(b,a);den=sum(x*x for x in ab)
                t=max(0.,min(1.,sum(x*y for x,y in zip(sub(p,a),ab))/den)) if den else 0.
                gap=norm(sub(p,lerp(a,b,t)))-capsule['radius']
                max_contact=max(max_contact,thickness-gap)
    def at(pid,point):
        for world,rest in by_piece[pid]:
            a,b=sub(rest[1],rest[0]),sub(rest[2],rest[0]);d=sub(point,rest[0]);det=a[0]*b[1]-a[1]*b[0]
            u=(d[0]*b[1]-d[1]*b[0])/det;v=(a[0]*d[1]-a[1]*d[0])/det
            if u>=-1e-8 and v>=-1e-8 and u+v<=1+1e-8:return [world[0][i]+u*(world[1][i]-world[0][i])+v*(world[2][i]-world[0][i]) for i in range(3)]
        raise AssertionError('authored seam sample missing from exported material-space mesh')
    points={p['id']:p['position'] for p in project['points']};curves={c['id']:c for c in project['curves']};owners={c:p['id'] for p in project['pieces'] for c in p['boundary']}
    def seam_position(interval,fraction):
        curve=curves[interval['curve']];assert curve['controls'] is None,'this independent fixture evaluator supports straight authored seams only'
        t=interval['t0']+(interval['t1']-interval['t0'])*fraction
        return at(owners[curve['id']],lerp(points[curve['start']],points[curve['end']],t))
    max_seam=0.
    for seam in project['seams']:
        pairs=seam['correspondence']
        for i in range(65):
            a=i/64.;b=None
            for first,second in zip(pairs,pairs[1:]):
                if first[0]<=a<=second[0]:b=first[1]+(second[1]-first[1])*(a-first[0])/(second[0]-first[0]);break
            assert b is not None,'incomplete seam correspondence'
            max_seam=max(max_seam,norm(sub(seam_position(seam['a'],a),seam_position(seam['b'],b))))
    # World bounds establish the metre-scale fixture; exact drape positions are not reference answers.
    bounds=[[min(p[i] for p in vertices),max(p[i] for p in vertices)] for i in range(3)]
    assert .5<bounds[0][1]-bounds[0][0]<1.5 and .2<bounds[1][1]-bounds[1][0]<1.5,'garment scale outside declared metre bounds'
    pattern_report=None
    if svg_path:
        svg=ET.parse(svg_path).getroot()
        assert svg.tag=='{http://www.w3.org/2000/svg}svg'
        assert all('transform' not in item.attrib and 'style' not in item.attrib for item in svg.iter()),'unassessed SVG transform/style'
        width,height=[float(svg.attrib[key].removesuffix('mm')) for key in ['width','height']]
        assert all(svg.attrib[key].endswith('mm') for key in ['width','height'])
        box=list(map(float,re.split('[ ,]+',svg.attrib['viewBox'])))
        assert len(box)==4 and all(math.isfinite(v) for v in box) and width>0 and height>0
        assert abs(width-box[2])<1e-6 and abs(height-box[3])<1e-6,'SVG physical scale is not one millimetre per user unit'
        paths=svg.findall('{http://www.w3.org/2000/svg}path')
        assert len(paths)==len(pieces)
        found=set()
        for path in paths:
            pid=int(path.attrib['id'].removeprefix('piece-'));assert pid in pieces and pid not in found;found.add(pid)
            boundary=[curves[c] for c in pieces[pid]['boundary']]
            assert all(c['controls'] is None for c in boundary),'straight fixture evaluator only'
            expected=[points[boundary[0]['start']]]+[points[c['end']] for c in boundary]
            expected=[[x*1000,-y*1000] for x,y in expected]
            d=path.attrib['d'];assert d.strip().endswith('Z') and not re.sub(r'[MLZ0-9eE+.,\s-]','',d),'unsupported path command'
            numbers=list(map(float,re.findall(r'[-+]?(?:\d*\.\d+|\d+\.?\d*)(?:[eE][-+]?\d+)?',d)))
            assert len(numbers)==len(expected)*2
            for actual,wanted in zip(zip(numbers[::2],numbers[1::2]),expected):
                assert all(abs(a-b)<1e-6 for a,b in zip(actual,wanted)),'SVG path differs from authored millimetre dimensions'
                assert box[0]-1e-6<=actual[0]<=box[0]+box[2]+1e-6 and box[1]-1e-6<=actual[1]<=box[1]+box[3]+1e-6,'SVG clips the physical pattern'
        pattern_report={'paths':len(paths),'millimetres_per_user_unit':[width/box[2],height/box[3]],'authored_points_match':True}
    passed=max_strain<.15 and max_seam<.005 and max_contact<.002
    return {'kind':'independently_checked_output','status':'passed' if passed else 'failed','scope':'straight-boundary original fixture; no self-contact or animation verification','vertices':len(vertices),'triangles':len(faces),'pieces':len(by_piece),'max_material_axis_strain':max_strain,'max_seam_gap_m':max_seam,'max_avatar_ground_thickness_error_m':max_contact,'bounds_m':bounds,'patterns':pattern_report,'criteria':{'strain_below':.15,'seam_gap_below_m':.005,'avatar_ground_thickness_error_below_m':.002}}
if __name__=='__main__':
    report=measure(*sys.argv[1:]);print(json.dumps(report,indent=2));sys.exit(0 if report['status']=='passed' else 1)
