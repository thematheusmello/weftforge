"""Independent energy/force/momentum evaluation from original geometry and saved public states."""
import math
from collections import defaultdict
from measure import dot, sub, measure, residuals


def add(a, b):
    return [x+y for x,y in zip(a,b)]


def mul(a, s):
    return [x*s for x in a]


def cross(a, b):
    return [a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0]]


def norm(a):
    return math.sqrt(dot(a,a))


def materials(meta):
    pieces={p['id']:p for p in meta['project']['pieces']}
    mats={m['id']:m for m in meta['project']['materials']}
    return pieces,mats


def masses(meta):
    pieces,mats=materials(meta)
    result=[0.]*len(meta['uv_m'])
    for tri in meta['triangles']:
        a,b,c=[meta['uv_m'][v] for v in tri]
        u,v=sub(b,a),sub(c,a)
        area=abs(u[0]*v[1]-u[1]*v[0])/2
        m=area*mats[pieces[meta['piece_ids'][tri[0]]]['material']]['areal_density']/3
        for node in tri:
            result[node]+=m
    return result


def membrane(meta, positions):
    pieces,mats=materials(meta)
    energy=0.
    force=[[0.,0.,0.] for _ in positions]
    for tri in meta['triangles']:
        piece=pieces[meta['piece_ids'][tri[0]]]
        mat=mats[piece['material']]
        a,b,c=[meta['uv_m'][v] for v in tri]
        u,v=sub(b,a),sub(c,a)
        det=u[0]*v[1]-u[1]*v[0]
        assert abs(det)>1e-12
        # Shape-function derivatives independently follow the 2x2 material edge inverse.
        du=[(u[1]-v[1])/det,v[1]/det,-u[1]/det]
        dv=[(v[0]-u[0])/det,-v[0]/det,u[0]/det]
        angle=piece['grain_angle']
        co,si=math.cos(angle),math.sin(angle)
        cw=[co*x+si*y for x,y in zip(du,dv)]
        cf=[-si*x+co*y for x,y in zip(du,dv)]
        relative=[sub(positions[j],positions[tri[0]]) for j in tri]
        fw=[sum(cw[j]*relative[j][k] for j in range(3)) for k in range(3)]
        ff=[sum(cf[j]*relative[j][k] for j in range(3)) for k in range(3)]
        strains=[(dot(fw,fw)-1)/2,(dot(ff,ff)-1)/2,dot(fw,ff)]
        stiffness=[mat['warp_stiffness'],mat['weft_stiffness'],mat['shear_stiffness']]
        area=abs(det)/2
        energy+=sum(area*k*c*c/2 for k,c in zip(stiffness,strains))
        for local,node in enumerate(tri):
            gradient=[add(mul(fw,cw[local]),[0.,0.,0.]),mul(ff,cf[local]),add(mul(ff,cw[local]),mul(fw,cf[local]))]
            for k,c,g in zip(stiffness,strains,gradient):
                force[node]=add(force[node],mul(g,-area*k*c))
    return energy,force


def angle(vertices, positions):
    a,b,c,d=[positions[v] for v in vertices]
    edge=sub(b,a)
    length=norm(edge)
    n1=cross(edge,sub(c,a))
    n2=cross(sub(d,a),edge)
    size1,size2=norm(n1),norm(n2)
    if length<1e-12 or size1<1e-14 or size2<1e-14:
        return None
    n1,n2=mul(n1,1/size1),mul(n2,1/size2)
    return math.atan2(dot(cross(n1,n2),mul(edge,1/length)),dot(n1,n2))


def bends(meta):
    pieces,mats=materials(meta)
    edges=defaultdict(list)
    for index,tri in enumerate(meta['triangles']):
        for i in range(3):
            edge=tuple(sorted([tri[i],tri[(i+1)%3]]))
            edges[edge].append((index,tri[(i+2)%3]))
    result=[]
    positions=meta['rest_positions_m']
    for edge,faces in sorted(edges.items()):
        if len(faces)!=2:
            continue
        (ti,c),(tj,d)=faces
        rigidities=[mats[pieces[meta['piece_ids'][meta['triangles'][t][0]]]['material']]['bending_stiffness'] for t in [ti,tj]]
        rigidity=sum(rigidities)/2
        if rigidity==0:
            continue
        a,b=[positions[n] for n in edge]
        e=sub(b,a)
        length=norm(e)
        h=(norm(cross(e,sub(positions[c],a)))+norm(cross(sub(positions[d],a),e)))/(2*length)
        vertices=[*edge,c,d]
        rest=angle(vertices,positions)
        assert rest is not None
        result.append((vertices,rest,rigidity*length/h))
    return result


def bend_energy_one(constraint, positions):
    vertices,rest,stiffness=constraint
    current=angle(vertices,positions)
    if current is None:
        return None
    delta=math.atan2(math.sin(current-rest),math.cos(current-rest))
    return stiffness*delta*delta/2


def bending(constraints, positions, forces=False, epsilon=1e-6):
    energy=0.
    force=[[0.,0.,0.] for _ in positions]
    degenerate=0
    for constraint in constraints:
        value=bend_energy_one(constraint,positions)
        if value is None:
            degenerate+=1
            continue
        energy+=value
        if forces:
            for node in constraint[0]:
                for axis in range(3):
                    old=positions[node][axis]
                    positions[node][axis]=old+epsilon
                    plus=bend_energy_one(constraint,positions)
                    positions[node][axis]=old-epsilon
                    minus=bend_energy_one(constraint,positions)
                    positions[node][axis]=old
                    assert plus is not None and minus is not None
                    force[node][axis]-=(plus-minus)/(2*epsilon)
    return energy,force,degenerate


def weighted(endpoint, positions):
    a,b=[positions[n] for n in endpoint['vertices']]
    return add(mul(a,1-endpoint['weight']),mul(b,endpoint['weight']))


def sewing(meta, positions, time_s):
    energy=0.
    force=[[0.,0.,0.] for _ in positions]
    for stitch in meta['stitches']:
        initial=norm(sub(weighted(stitch['a'],meta['rest_positions_m']),weighted(stitch['b'],meta['rest_positions_m'])))
        target=max(0.,initial-time_s)
        delta=sub(weighted(stitch['a'],positions),weighted(stitch['b'],positions))
        length=norm(delta)
        c=length-target
        compliance=stitch.get('compliance_m_per_N',1e-8)
        energy+=c*c/(2*compliance)
        if length>1e-14:
            force_a=mul(delta,-c/(compliance*length))
            for end,sign in [(stitch['a'],1),(stitch['b'],-1)]:
                for node,weight in zip(end['vertices'],[1-end['weight'],end['weight']]):
                    force[node]=add(force[node],mul(force_a,sign*weight))
    return energy,force


def dynamics(meta, positions, velocity):
    mass=masses(meta)
    total=sum(mass)
    p=[sum(m*v[k] for m,v in zip(mass,velocity)) for k in range(3)]
    com=[sum(m*x[k] for m,x in zip(mass,positions))/total for k in range(3)]
    angular=[0.,0.,0.]
    for m,x,v in zip(mass,positions,velocity):
        angular=add(angular,cross(sub(x,com),mul(v,m)))
    kinetic=sum(m*dot(v,v)/2 for m,v in zip(mass,velocity))
    return {'mass_kg':total,'center_of_mass_m':com,'momentum_kg_m_per_s':p,'angular_momentum_about_com_kg_m2_per_s':angular,'kinetic_energy_J':kinetic,'rms_speed_m_per_s':math.sqrt(2*kinetic/total),'max_speed_m_per_s':max(map(norm,velocity))}


def self_check():
    meta={'project':{'pieces':[{'id':1,'material':2,'grain_angle':0.}],'materials':[{'id':2,'areal_density':.2,'warp_stiffness':50.,'weft_stiffness':1000.,'shear_stiffness':100.,'bending_stiffness':.003}]},'piece_ids':[1,1,1],'triangles':[[0,1,2]],'uv_m':[[0.,0.],[1.,0.],[0.,1.]],'rest_positions_m':[[0.,0.,0.],[1.,0.,0.],[0.,1.,0.]],'stitches':[]}
    pos=[[0.,0.,0.],[1.2,0.,0.],[0.,1.,0.]]
    e,f=membrane(meta,pos)
    assert abs(e-.5*50*.22**2/2)<1e-14
    assert abs(f[1][0]+6.6)<1e-13
    for node in range(3):
        for axis in range(3):
            old=pos[node][axis]
            eps=1e-6
            pos[node][axis]=old+eps
            ep=membrane(meta,pos)[0]
            pos[node][axis]=old-eps
            em=membrane(meta,pos)[0]
            pos[node][axis]=old
            assert abs((ep-em)/(2*eps)+f[node][axis])<1e-7
    assert abs(sum(masses(meta))-.1)<1e-15
    state=dynamics(meta,pos,[[2.,0.,0.]]*3)
    assert abs(state['kinetic_energy_J']-.2)<1e-15
    assert abs(state['momentum_kg_m_per_s'][0]-.2)<1e-15
    meta['stitches']=[{'a':{'vertices':[0,0],'weight':0.},'b':{'vertices':[1,1],'weight':0.},'compliance_m_per_N':.1}]
    se,sf=sewing(meta,pos,1.)
    assert abs(se-7.2)<1e-14
    assert abs(sf[0][0]-12.)<1e-14
    assert abs(sf[1][0]+12.)<1e-14
    flat=[[0.,0.,0.],[1.,0.,0.],[0.,1.,0.],[0.,-1.,0.]]
    folded=[p[:] for p in flat]
    folded[3][2]=.5
    constraint=([0,1,2,3],angle([0,1,2,3],flat),.003)
    be,bf,degenerate=bending([constraint],folded,True)
    assert be>0 and degenerate==0
    assert norm([sum(v[k] for v in bf) for k in range(3)])<1e-10
    assert bf[3][2]<0
    print('Independent membrane force/energy finite differences, physical mass, momentum, sewing and bending self-checks passed.')


if __name__=='__main__':
    self_check()
