#!/usr/bin/env python3
"""Independent standard-library checks; does not use WeftForge's parser/writer."""
import math
import pathlib
import re
import sys
import xml.etree.ElementTree as ET

NUMBER = r'[-+]?(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][-+]?\d+)?'

def check_svg(svg_path):
    """Check the original straight-edged shirt fixture, not arbitrary SVG files."""
    svg=ET.parse(svg_path).getroot()
    assert svg.tag=='{http://www.w3.org/2000/svg}svg'
    dimensions=[svg.attrib[key] for key in ('width','height')]
    assert all(value.endswith('mm') for value in dimensions),'physical units missing'
    physical=[float(value[:-2]) for value in dimensions]
    viewbox=list(map(float,re.split('[ ,]+',svg.attrib['viewBox'].strip())))
    assert len(viewbox)==4 and all(math.isfinite(v) for v in physical+viewbox),'invalid SVG dimensions'
    assert all(v>0 for v in physical+viewbox[2:]),'SVG dimensions must be positive'
    assert all(abs(a-b)<1e-6 for a,b in zip(physical,viewbox[2:])),'SVG must map one user unit to one physical millimetre (1:1)'
    # The original fixture's page includes the origin; pattern bounds exclude that margin.
    assert all(abs(a-b)<1e-6 for a,b in zip(viewbox,[-530.,-1560.,2260.,1560.])),'original shirt page bounds changed'
    assert all('transform' not in node.attrib and 'style' not in node.attrib for node in svg.iter()),'unhandled SVG transform/style'
    assert not svg.findall('.//{http://www.w3.org/2000/svg}style'),'unhandled SVG stylesheet'
    paths=svg.findall('.//{http://www.w3.org/2000/svg}path')
    assert len(paths)==2,'two original pattern paths required'
    bounds=[]
    for path in paths:
        data=path.attrib['d']
        assert re.fullmatch(rf'\s*M\s*{NUMBER}[\s,]+{NUMBER}(?:\s*L\s*{NUMBER}[\s,]+{NUMBER})+\s*Z\s*',data),'original shirt requires closed absolute line paths'
        coordinates=list(map(float,re.findall(NUMBER,data)))
        assert all(math.isfinite(v) for v in coordinates),'non-finite pattern coordinates'
        x=coordinates[::2];y=coordinates[1::2]
        bounds.append([min(x),min(y),max(x),max(y)])
    expected=[[-530.,-1560.,530.,-1020.],[670.,-1560.,1730.,-1020.]]
    assert all(abs(a-b)<1e-6 for actual,wanted in zip(sorted(bounds),expected) for a,b in zip(actual,wanted)),'original shirt pattern bounds must be millimetres'
    return len(paths)

def check(obj_path,svg_path):
    vertices=[];uv=[];normals=[];faces=[];groups=[]
    for line in pathlib.Path(obj_path).read_text().splitlines():
        values=line.split()
        if not values:continue
        if values[0]=='v':vertices.append(tuple(map(float,values[1:])))
        if values[0]=='vt':uv.append(tuple(map(float,values[1:])))
        if values[0]=='vn':normals.append(tuple(map(float,values[1:])))
        if values[0]=='f':faces.append([tuple(map(int,v.split('/'))) for v in values[1:]])
        if values[0]=='g':groups.append(values[1:])
    assert vertices and uv and normals and faces and groups,'required OBJ attributes missing'
    assert all(len(v)==3 and all(math.isfinite(n) for n in v) for v in vertices)
    assert all(len(t)==3 and all(1<=v<=len(vertices) and 1<=u<=len(uv) and 1<=n<=len(normals) for v,u,n in t) for t in faces)
    assert all(abs(sum(c*c for c in n)-1)<1e-5 for n in normals),'normals must be unit length'
    # Original shirt expected bounds declared by original authored dimensions, before drape.
    assert abs(min(p[0] for p in vertices)+0.53)<1e-8
    assert abs(max(p[0] for p in vertices)-0.53)<1e-8
    assert abs(min(p[1] for p in vertices)-1.02)<1e-8
    assert abs(max(p[1] for p in vertices)-1.56)<1e-8
    assert abs(min(p[2] for p in vertices)+0.22)<1e-8
    assert abs(max(p[2] for p in vertices)-0.22)<1e-8
    paths=check_svg(svg_path)
    print(f'Independent OBJ/SVG checks passed: {len(vertices)} vertices, {len(faces)} triangles, {paths} closed patterns; metres/mm, Y-up, UV, normals, piece groups.')
if __name__=='__main__':check(*sys.argv[1:])
