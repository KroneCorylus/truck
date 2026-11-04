"""Input-derived whole A5 boundary, flux and ray crossings (no result geometry)."""
import itertools
import json
import math
from pathlib import Path
import numpy as np


def unit(v):
    return v / np.linalg.norm(v)


def key(p):
    return tuple(np.round(p, 11))


def area(p):
    return sum(np.cross(a, b) for a, b in zip(p, np.roll(p, -1, axis=0))) / 2


def build(mode):
    data = json.loads(Path(__file__).with_name('a5_da_assembly_input.json' if mode == 'dist_angle' else 'a5_assembly_input.json').read_text())
    edges = data['modes'][mode]
    faces = data['faces']
    normals = [None if f['normal'] is None else np.array(f['normal']) for f in faces]
    ends = np.array([e['ends'] for e in edges])
    contacts = np.array([[e['ends'], e['ends']] for e in edges])
    incident = {}
    for k, e in enumerate(ends):
        for end, v in enumerate(e):
            incident.setdefault(key(v), []).append((k, end))
    planes = []
    for f, face in enumerate(faces):
        k, rev = face['wires'][0][0]
        planes.append(None if normals[f] is None else (normals[f], normals[f] @ ends[k, int(rev)]))
    bevel = {}
    transverse = {}
    for k, e in enumerate(edges):
        if e['distances'] is None or not e['line']:
            continue
        p = []
        for f, setback in zip(e['faces'], e['distances']):
            rev = next(rev for wire in faces[f]['wires'] for j, rev in wire if j == k)
            direction = unit(ends[k, int(not rev)] - ends[k, int(rev)])
            inward = unit(np.cross(normals[f], direction))
            transverse[k, f] = inward
            p.append(ends[k, 0] + setback * inward)
        n = unit(np.cross(p[1] - p[0], ends[k, 1] - ends[k, 0]))
        if n @ sum(normals[f] for f in e['faces']) < 0:
            n = -n
        bevel[k] = (n, n @ p[0])
    def common(a, b):
        return next(f for f in edges[a]['faces'] if f in edges[b]['faces'])
    def meet(equations):
        return np.linalg.solve([x[0] for x in equations], [x[1] for x in equations])
    def set_contact(k, f, v, point):
        end = int(key(ends[k, 1]) == key(v))
        contacts[k, edges[k]['faces'].index(f), end] = point
    corner_extra = {}
    patches = []
    cx, cy = data['cylinder']['center']
    radius = data['cylinder']['radius']
    for entries in incident.values():
        chosen = [k for k, _ in entries if edges[k]['distances'] is not None]
        if not chosen:
            continue
        assert len(entries) == 3
        v = ends[entries[0][0], entries[0][1]]
        face_ids = set(f for k, _ in entries for f in edges[k]['faces'])
        if 17 in face_ids:
            cap = next(f for f in face_ids if f != 17 and abs(normals[f][2]) > .9)
            side = next(f for f in face_ids if f not in (17, cap))
            rim = next(k for k in chosen if not edges[k]['line'])
            straight = next(k for k in chosen if k != rim)
            seam = next(k for k, _ in entries if k not in chosen)
            setbacks = dict(zip(edges[rim]['faces'], edges[rim]['distances']))
            radial = unit(np.array([v[0] - cx, v[1] - cy, 0.]))
            t, u = v - setbacks[cap] * radial, v - setbacks[17] * normals[cap]
            for k, f, p in [(rim, cap, t), (straight, cap, t), (rim, 17, u), (straight, side, u), (seam, 17, u), (seam, side, u)]:
                set_contact(k, f, v, p)
            continue
        if len(chosen) == 1:
            a = chosen[0]
            for k, _ in entries:
                if k == a:
                    continue
                fs = edges[k]['faces']
                p = meet([planes[f] for f in fs] + [bevel[a]])
                for f in fs:
                    set_contact(k, f, v, p)
                    if f in edges[a]['faces']:
                        set_contact(a, f, v, p)
            continue
        concave = {k: transverse[k, edges[k]['faces'][0]] @ normals[edges[k]['faces'][1]] > 0 for k in chosen}
        if len(chosen) == 2:
            a, b = chosen
            f = common(a, b)
            other = next(k for k, _ in entries if k not in chosen)
            roots = {k: meet([planes[g] for g in edges[other]['faces']] + [bevel[k]]) for k in chosen}
            tip = meet([planes[f], bevel[a], bevel[b]])
            if np.linalg.norm(roots[a]-roots[b]) >= 1e-8:
                assert not any(concave.values())
                deep=max(chosen,key=lambda k:np.linalg.norm(roots[k]-v))
                shallow=next(k for k in chosen if k!=deep)
                terminal=next(g for g in edges[shallow]['faces'] if g!=f)
                joint=meet([planes[terminal],bevel[a],bevel[b]])
                # Both bevel inequalities constrain the unselected ray. The farther
                # intercept survives; the shallower support meets the other bevel.
                for k in chosen:
                    for g in edges[k]['faces']:
                        set_contact(k,g,v,tip if g==f else roots[deep] if k==deep else joint)
                for g in edges[other]['faces']:set_contact(other,g,v,roots[deep])
                end=int(key(ends[deep,1])==key(v));corner_extra[deep,end]=[joint]
                continue
            for k in chosen:
                for g in edges[k]['faces']:
                    set_contact(k, g, v, tip if g == f else roots[k])
                    if g != f:
                        set_contact(other, g, v, roots[k])
            continue
        away = {k: unit(ends[k, 1 - end] - v) for k, end in entries}
        crossing = {tuple(sorted((a, b))): meet([planes[common(a, b)], bevel[a], bevel[b]]) for a, b in itertools.combinations(chosen, 2)}
        def cross(a, b):
            return crossing[tuple(sorted((a, b)))]
        candidates = []
        for p in chosen:
            a, b = [k for k in chosen if k != p]
            A, B, Q = cross(p, a), cross(p, b), cross(a, b)
            alpha, beta = (A - Q) @ away[a], (B - Q) @ away[b]
            D, C = Q + alpha * away[a], Q + beta * away[b]
            ratio = np.linalg.norm(C - D) / max(np.linalg.norm(C - A), np.linalg.norm(D - B))
            candidates.append(dict(pivot=p, a=a, b=b, depths=[alpha, beta], ratio=ratio, points=np.array([A, B, C, D])))
        concaves = [k for k in chosen if concave[k]]
        plan = None
        if len(concaves) == 1:
            plan = next(p for p in candidates if p['pivot'] == concaves[0])
        elif len(concaves) == 0:
            valid = [p for p in candidates if min(p['depths']) >= -1e-9]
            sections = [p for p in valid if p['ratio'] > .3]
            if len(valid) == 1 and sections:
                plan = sections[0]
            elif sections:
                # Explicit fixture preference, bound to original vertex/edge geometry.
                preferred = json.loads(Path(__file__).with_name('a5_assembly_preference.json').read_text())
                assert np.linalg.norm(v - preferred['vertex']) < 1e-9
                wanted = [k for k in chosen if all(any(np.linalg.norm(p-q)<1e-9 for q in ends[k]) for p in preferred['edge']['points'])]
                assert len(wanted) == 1
                plan = next(p for p in sections if p['pivot'] == wanted[0])
        else:
            assert len(concaves) == 3
        if mode == 'dist_angle':
            preferences = json.loads(Path(__file__).with_name('a5_da_assembly_preference.json').read_text())
            nominated = [p for p in preferences if np.linalg.norm(v - p['vertex']) < 1e-9]
            if nominated:
                assert len(nominated) == 1
                wanted = [k for k in chosen if all(any(np.linalg.norm(p-q)<1e-9 for q in ends[k]) for p in nominated[0]['edge']['points'])]
                assert len(wanted) == 1
                choice = next(p for p in candidates if p['pivot'] == wanted[0])
                plan = choice if choice['ratio'] > .3 else None
            elif len(concaves) == 3:
                valid = [p for p in candidates if min(p['depths']) > 1e-9]
                assert len(valid) == 1 and valid[0]['ratio'] > .3
                plan = valid[0]
        for a, b in itertools.combinations(chosen, 2):
            f = common(a, b)
            for k in (a, b):
                set_contact(k, f, v, cross(a, b))
        if plan is None:
            a, b, c = chosen
            polygon = np.array([cross(a, b), cross(b, c), cross(c, a)])
        else:
            polygon = plan['points']
            a, b = plan['a'], plan['b']
            set_contact(a, common(a, b), v, polygon[3])
            set_contact(b, common(a, b), v, polygon[2])
        outward = sum(bevel[k][0] for k in chosen)
        if area(polygon) @ outward < 0:
            polygon = polygon[::-1]
        patches.append(dict(kind='bilinear' if len(polygon)==4 else 'polygon', points=polygon.tolist()))
    polygons = []
    for f, face in enumerate(faces):
        if f == 17:
            continue
        loops = []
        for wire in face['wires']:
            segments = []
            for k, reverse in wire:
                pair = contacts[k, edges[k]['faces'].index(f)]
                if reverse:
                    pair = pair[::-1]
                segments.append(dict(a=pair[0].tolist(), b=pair[1].tolist(), arc=not edges[k]['line']))
            closed = []
            for i, segment in enumerate(segments):
                closed.append(segment)
                a, b = segment['b'], segments[(i+1)%len(segments)]['a']
                if np.linalg.norm(np.array(a)-b)>1e-10:
                    closed.append(dict(a=a,b=b,arc=False))
            loops.append(closed)
        polygons.append(dict(normal=normals[f].tolist(), loops=loops, source=f))
    for k, (n, _) in bevel.items():
        a, b = contacts[k, 0]
        c, d = contacts[k, 1]
        p = np.array([a, b, *corner_extra.get((k,1),[]), d, c, *corner_extra.get((k,0),[])])
        if area(p) @ n < 0:
            p = p[::-1]
        polygons.append(dict(normal=n.tolist(),loops=[[dict(a=a.tolist(),b=b.tolist(),arc=False) for a,b in zip(p,np.roll(p,-1,axis=0))]],bevel=k))
    for patch in patches:
        if patch['kind']=='polygon':
            p=np.array(patch['points']);n=unit(area(p))
            polygons.append(dict(normal=n.tolist(),loops=[[dict(a=a.tolist(),b=b.tolist(),arc=False) for a,b in zip(p,np.roll(p,-1,axis=0))]]))
    patches=[p['points'] for p in patches if p['kind']=='bilinear']
    conics = [k for k,e in enumerate(edges) if not e['line']]
    arcs = []
    surfaces = []
    for k in conics:
        cap = next(f for f in edges[k]['faces'] if f != 17)
        flat = contacts[k, edges[k]['faces'].index(cap)]
        cyl = contacts[k, edges[k]['faces'].index(17)]
        assert abs(flat[0,2]-flat[1,2])<1e-9 and abs(cyl[0,2]-cyl[1,2])<1e-9
        angles=sorted(math.atan2(p[1]-cy,p[0]-cx) for p in ends[k])
        radii=[np.linalg.norm(flat[0,:2]-[cx,cy]),radius]
        z=[flat[0,2],cyl[0,2]]
        if z[0]>z[1]:z.reverse();radii.reverse()
        surfaces.append(dict(z=z,radii=radii,angles=angles))
        arcs.append(cyl[0,2])
    surfaces.append(dict(z=sorted(arcs),radii=[radius,radius],angles=surfaces[0]['angles']))
    return dict(mode=mode,center=[cx,cy],polygons=polygons,bilinear=patches,radial=surfaces,contacts=contacts.tolist())


def flux(model):
    value=0.; records=[];center=np.array([*model['center'],0.])
    for face in model['polygons']:
        n=np.array(face['normal']);integral=np.zeros(3)
        for loop in face['loops']:
            for edge in loop:
                a,b=np.array(edge['a']),np.array(edge['b'])
                if edge['arc']:
                    c=center.copy();c[2]=a[2];u,v=a-c,b-c
                    angle=math.atan2(np.cross(u,v)[2],u@v)
                    integral+=np.cross(c,b-a)+np.array([0.,0.,u@u*angle])
                else:integral+=np.cross(a,b)
        a=np.array(face['loops'][0][0]['a']);part=(n@a)*(n@integral)/6
        value+=part;records.append(float(part))
    for patch in model['bilinear']:
        A,B,C,D=np.array(patch);part=0.
        for u,v in itertools.product([.5-.5/math.sqrt(3),.5+.5/math.sqrt(3)],repeat=2):
            p=(1-u)*(1-v)*A+u*(1-v)*B+u*v*C+(1-u)*v*D
            du=(1-v)*(B-A)+v*(C-D);dv=(1-u)*(D-A)+u*(C-B)
            part+=p@np.cross(du,dv)/12
        value+=part;records.append(float(part))
    cx,cy=model['center']
    for face in model['radial']:
        a,b=face['angles'];z0,z1=face['z'];r0,r1=face['radii'];dz=z1-z0;dr=r1-r0
        angular=cx*(math.sin(b)-math.sin(a))+cy*(math.cos(a)-math.cos(b));part=0.
        for t in [.5-.5/math.sqrt(3),.5+.5/math.sqrt(3)]:
            rho=r0+t*dr;z=z0+t*dz
            part+=rho*(dz*angular+(b-a)*(dz*rho-dr*z))/6
        value+=part;records.append(float(part))
    return value,records


def incidence(model):
    segments=[]
    for face in model['polygons']:
        for loop in face['loops']:
            segments.extend((np.array(e['a']),np.array(e['b']),e['arc']) for e in loop)
    for patch in model['bilinear']:
        p=np.array(patch);segments.extend((a,b,False) for a,b in zip(p,np.roll(p,-1,axis=0)))
    cx,cy=model['center']
    for face in model['radial']:
        a,b=face['angles'];z0,z1=face['z'];r0,r1=face['radii']
        p=np.array([[cx+r*math.cos(t),cy+r*math.sin(t),z] for t,r,z in [(a,r0,z0),(b,r0,z0),(b,r1,z1),(a,r1,z1)]])
        segments.extend((p[i],p[(i+1)%4],i%2==0) for i in range(4))
    vertices=[p for a,b,_ in segments for p in [a,b]];uses={}
    for a,b,arc in segments:
        if np.linalg.norm(b-a)<1e-10:continue
        points=[(0.,a),(1.,b)]
        if not arc:
            axis=b-a;length2=axis@axis
            for p in vertices:
                t=(p-a)@axis/length2
                if 1e-9<t<1-1e-9 and np.linalg.norm(p-a-t*axis)<1e-9:points.append((t,p))
        points=sorted(points,key=lambda x:x[0]);unique=[]
        for t,p in points:
            q=tuple(np.round(p,9))
            if not unique or q!=unique[-1]:unique.append(q)
        for x,y in zip(unique,unique[1:]):
            edge=(arc,*sorted([x,y]));uses.setdefault(edge,[]).append(1 if x<y else -1)
    bad=[dict(edge=k,uses=v) for k,v in uses.items() if sorted(v)!=[-1,1]]
    return dict(edges=len(uses),bad_edges=bad)


def inside_face(face, points, center):
    normal=np.array(face['normal']);drop=int(np.argmax(abs(normal)));ids=[i for i in range(3) if i!=drop]
    h=points[:,ids];inside=np.zeros(len(points),bool);bad=np.zeros(len(points),bool)
    for loop in face['loops']:
        for edge in loop:
            a,b=np.array(edge['a']),np.array(edge['b'])
            if edge['arc']:
                assert drop==2
                c=np.array(center);rho=np.linalg.norm(a[:2]-c);start=math.atan2(a[1]-c[1],a[0]-c[0]);end=math.atan2(b[1]-c[1],b[0]-c[0]);angle=(end-start+math.pi)%(2*math.pi)-math.pi
                radicand=rho*rho-(h[:,1]-c[1])**2
                for sign in [-1,1]:
                    x=c[0]+sign*np.sqrt(np.maximum(radicand,0));theta=np.arctan2(h[:,1]-c[1],x-c[0]);span=((theta-start)*np.sign(angle))%(2*math.pi)
                    hit=(radicand>0)&(span<abs(angle))&(x>h[:,0]);inside ^= hit
                    bad |= hit&((span<1e-9)|(abs(span-abs(angle))<1e-9)|(abs(radicand)<1e-12))
            else:
                a,b=a[ids],b[ids];delta=b-a;rel=h-a
                if abs(delta[1])>1e-15:
                    inside ^= ((a[1]>h[:,1])!=(b[1]>h[:,1]))&(h[:,0]<a[0]+(h[:,1]-a[1])*delta[0]/delta[1])
                cross=delta[0]*rel[:,1]-delta[1]*rel[:,0]
                bad |= (abs(cross)<1e-10)&(rel@delta>=-1e-10)&(rel@delta<=delta@delta+1e-10)
    return inside,bad


def crossings(model, points, direction):
    points=np.array(points);d=np.array(direction);counts=np.zeros(len(points),int);bad=np.zeros(len(points),bool);events=[]
    def add(t,inside,normal_dot,uncertain):
        nonlocal counts,bad
        signs=np.sign(normal_dot)*np.ones(len(points))
        counts+=np.where(inside&(t>1e-10),signs,0).astype(int)
        bad |= uncertain | (inside & ((abs(t)<1e-10)|(abs(normal_dot)<1e-10)))
        events.append((np.where(inside,t,np.inf),signs))
    for face in model['polygons']:
        n=np.array(face['normal']);den=n@d;assert abs(den)>1e-10
        a=np.array(face['loops'][0][0]['a']);t=((a-points)@n)/den;hits=points+t[:,None]*d
        inside,edge=inside_face(face,hits,model['center']);add(t,inside,den,edge)
    cross2=lambda a,b:a[...,0]*b[...,1]-a[...,1]*b[...,0]
    for patch in model['bilinear']:
        A,B,C,D=np.array(patch);b=B-A;c=D-A;h=A-B+C-D
        project=lambda x:x[...,1:]-x[...,0,None]*d[1:]/d[0]
        R=project(points-A);bb,cc,hh=map(project,[b,c,h]);qa=-cross2(bb,hh);qb=cross2(R,hh)-cross2(bb,cc);qc=cross2(R,cc);dis=qb*qb-4*qa*qc
        roots=[-qc/qb] if abs(qa)<1e-14 else [(-qb+sign*np.sqrt(np.maximum(dis,0)))/(2*qa) for sign in [-1,1]]
        for u in roots:
            cv=cc+u[:,None]*hh;v=np.sum((R-u[:,None]*bb)*cv,axis=1)/np.sum(cv*cv,axis=1)
            q=A+u[:,None]*b+v[:,None]*c+(u*v)[:,None]*h;t=(q[:,0]-points[:,0])/d[0]
            den=np.cross(b+v[:,None]*h,c+u[:,None]*h)@d
            inside=(u>0)&(u<1)&(v>0)&(v<1)&(dis>=0)
            edge=(dis>=0)&(u>=-1e-9)&(u<=1+1e-9)&(v>=-1e-9)&(v<=1+1e-9)&((abs(u)<1e-9)|(abs(u-1)<1e-9)|(abs(v)<1e-9)|(abs(v-1)<1e-9))
            add(t,inside,den,edge)
    center=np.array(model['center'])
    for face in model['radial']:
        z0,z1=face['z'];r0,r1=face['radii'];slope=(r1-r0)/(z1-z0)
        xy=points[:,:2]-center;r=r0+slope*(points[:,2]-z0)
        qa=d[:2]@d[:2]-(slope*d[2])**2
        qb=2*(xy@d[:2]-r*slope*d[2]);qc=np.sum(xy*xy,axis=1)-r*r;dis=qb*qb-4*qa*qc
        assert abs(qa)>1e-12
        for sign in [-1,1]:
            t=(-qb+sign*np.sqrt(np.maximum(dis,0)))/(2*qa);q=points+t[:,None]*d
            theta=np.arctan2(q[:,1]-center[1],q[:,0]-center[0]);a,b=face['angles'];rho=r0+slope*(q[:,2]-z0)
            inside=(dis>0)&(q[:,2]>z0)&(q[:,2]<z1)&(theta>a)&(theta<b)
            den=((q[:,:2]-center)@d[:2])/rho-slope*d[2]
            edge=inside&((abs(q[:,2]-z0)<1e-9)|(abs(q[:,2]-z1)<1e-9)|(abs(theta-a)<1e-9)|(abs(theta-b)<1e-9)|(abs(dis)<1e-12))
            add(t,inside,den,edge)
    # Ordered crossings partition each complete fiber, starting at exterior.
    ts=np.array([x[0] for x in events]).T;signs=np.array([x[1] for x in events]).T;order=np.argsort(ts,axis=1)
    ordered=np.take_along_axis(ts,order,axis=1);steps=np.take_along_axis(signs,order,axis=1)
    states=np.cumsum(np.where(np.isfinite(ordered),-steps,0),axis=1)
    invalid=np.any((states!=0)&(states!=1),axis=1)|(states[:,-1]!=0)
    return counts,bad,invalid


def surface_witnesses(model):
    points=[]
    def pair(p,n):
        n=unit(n);points.extend([p-1e-4*n,p+1e-4*n])
    for face in model['polygons']:
        n=np.array(face['normal']);drop=int(np.argmax(abs(n)));ids=[i for i in range(3) if i!=drop]
        corners=np.array([edge['a'] for loop in face['loops'] for edge in loop]);lo=np.min(corners,axis=0);hi=np.max(corners,axis=0);h=n@corners[0]
        found=False
        for u,v in itertools.product([.5,.35,.65,.2,.8],repeat=2):
            p=np.zeros(3);p[ids]=lo[ids]+np.array([u,v])*(hi[ids]-lo[ids]);p[drop]=(h-n[ids]@p[ids])/n[drop]
            inside,bad=inside_face(face,np.array([p]),model['center'])
            if inside[0] and not bad[0]:pair(p,n);found=True;break
        if not found:
            # A transition triangle can miss the bounding-box lattice.
            assert len(face['loops'])==1 and len(face['loops'][0])==3
            pair(np.mean(corners,axis=0),n)
    for patch in model['bilinear']:
        A,B,C,D=np.array(patch)
        for u,v in itertools.product([.2,.5,.8],repeat=2):
            p=(1-u)*(1-v)*A+u*(1-v)*B+u*v*C+(1-u)*v*D
            pair(p,np.cross((1-v)*(B-A)+v*(C-D),(1-u)*(D-A)+u*(C-B)))
    cx,cy=model['center']
    for face in model['radial']:
        z0,z1=face['z'];r0,r1=face['radii'];a,b=face['angles'];slope=(r1-r0)/(z1-z0)
        for u,v in itertools.product([.2,.5,.8],repeat=2):
            angle=a+u*(b-a);rho=r0+v*(r1-r0);z=z0+v*(z1-z0)
            pair(np.array([cx+rho*math.cos(angle),cy+rho*math.sin(angle),z]),np.array([math.cos(angle),math.sin(angle),-slope]))
    return np.array(points),np.tile([1,0],len(points)//2)


if __name__=='__main__':
    import sys
    models=[build(mode) for mode in ['equal_dist','dist_dist']]
    for model in models:
        model['volume'],model['face_flux']=flux(model)
        print(model['mode'],model['volume'])
    if len(sys.argv)>1:Path(sys.argv[1]).write_text(json.dumps(models,indent=2))
