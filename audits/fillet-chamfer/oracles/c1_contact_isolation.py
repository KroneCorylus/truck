"""Bounded original-chart contact isolation; independent of Truck and native outputs.

Uses de Casteljau hull subdivision and interval Krawczyk inclusion at fixed
stations. Floating point hulls have an explicit margin, not exact certification.
Unresolved boxes (including boundary roots) never count as certified roots.
"""
import argparse
import hashlib
import json
import math
from pathlib import Path

MARGIN = 2e-12


def mix(a, b, t):
    return [x * (1 - t) + y * t for x, y in zip(a, b)]


def split(c, t=0.5):
    left, right = [c[0]], [c[-1]]
    while len(c) > 1:
        c = [mix(a, b, t) for a, b in zip(c, c[1:])]
        left.append(c[0]); right.append(c[-1])
    return left, right[::-1]


def restrict(c, a, b):
    c = split(c, b)[0] if b != 1 else c
    return split(c, a / b)[1] if a else c


def value(c, t):
    return split(c, t)[0][-1]


def derivative(c):
    return [[(y - x) * (len(c) - 1) for x, y in zip(a, b)]
            for a, b in zip(c, c[1:])]


def spans(knots, poles, degree):
    """Insert internal knots to degree multiplicity, preserving the input chart."""
    knots, poles = list(knots), list(poles)
    for t in sorted(set(knots))[1:-1]:
        while knots.count(t) < degree:
            k = max(i for i, v in enumerate(knots) if v == t)
            s = knots.count(t)
            q = poles[:k - degree + 1]
            for i in range(k - degree + 1, k - s + 1):
                q.append(mix(poles[i - 1], poles[i],
                             (t - knots[i]) / (knots[i + degree] - knots[i])))
            q.extend(poles[k - s:])
            poles = q; knots.insert(k + 1, t)
    unique = sorted(set(knots))
    return [(a, b, poles[i * degree:i * degree + degree + 1])
            for i, (a, b) in enumerate(zip(unique, unique[1:]))]


def patches(face):
    u0, u1, v0, v1 = face['bounds']
    if 'base_poles' in face:
        for a, b, c in spans(face['base_knots'], face['base_poles'], face['base_degree']):
            yield (a, b, v0, v1), [[
                [p[k] + v * face['direction'][k] for k in range(3)]
                for v in (v0, v1)] for p in c]
    else:
        nu, nv = face['degree']
        # Decompose V first, then U using flattened V rows as vector coefficients.
        rows = [spans(face['vknots'], row, nv) for row in face['poles']]
        for j in range(len(rows[0])):
            a, b, _ = rows[0][j]
            if b <= v0 or a >= v1:
                continue
            flat = [[x for p in row[j][2] for x in p] for row in rows]
            for c, d, grid in spans(face['uknots'], flat, nu):
                if d <= u0 or c >= u1:
                    continue
                grid = restrict(grid, (max(c, u0)-c)/(d-c), (min(d,u1)-c)/(d-c))
                net = [restrict([row[k:k+3] for k in range(0,len(row),3)],
                                (max(a,v0)-a)/(b-a), (min(b,v1)-a)/(b-a)) for row in grid]
                yield (max(c,u0),min(d,u1),max(a,v0),min(b,v1)), net


def hull(values):
    return min(values)-MARGIN, max(values)+MARGIN


def add(a,b): return a[0]+b[0], a[1]+b[1]
def mul(a,b):
    p = [x*y for x in a for y in b]
    return min(p),max(p)
def scale(a,s): return mul(a,(s,s))
def square(a):
    return (0 if a[0] <= 0 <= a[1] else min(x*x for x in a), max(x*x for x in a))
def dot(a,b):
    r=(0,0)
    for x,y in zip(a,b): r=add(r,mul(x,y))
    return r

def net_hull(net):
    return [hull([p[k] for row in net for p in row]) for k in range(3)]

def evaluate(net): return value([value(row,.5) for row in net],.5)

def isolate(net, domain, guide, tangent, distance, max_depth=32, max_boxes=50000):
    stack=[(net,domain,0)]; roots=[]; unresolved=[]; excluded=0; visited=0; leaves=[]; k_exclusions=[]
    while stack:
        n, box, depth=stack.pop(); visited+=1
        if visited > max_boxes:
            unresolved.extend([box]+[b for _,b,_ in stack]); break
        if not all(math.isfinite(x) for row in n for p in row for x in p):
            unresolved.append(box); continue
        h=net_hull(n); delta=[add(x,(-g,-g)) for x,g in zip(h,guide)]
        f0=dot(delta,[(v,v) for v in tangent])
        f1=add(tuple(map(sum,zip(*(square(x) for x in delta)))),(-distance**2,-distance**2))
        if any(a > MARGIN or b < -MARGIN for a,b in (f0,f1)):
            excluded+=1; leaves.append({'box':box,'equation_bounds':[f0,f1]}); continue
        du=list(map(list,zip(*[derivative(list(col)) for col in zip(*n)])))
        dv=[derivative(row) for row in n]
        j=[[dot(net_hull(d),[(v,v) for v in tangent]) for d in (du,dv)],
           [scale(dot(delta,net_hull(d)),2) for d in (du,dv)]]
        p=evaluate(n); e=[x-y for x,y in zip(p,guide)]
        f=[sum(x*y for x,y in zip(e,tangent)),sum(x*x for x in e)-distance**2]
        mid=[[sum(x)/2 for x in row] for row in j]
        det=mid[0][0]*mid[1][1]-mid[0][1]*mid[1][0]
        if det and math.isfinite(det):
            inv=[[mid[1][1]/det,-mid[0][1]/det],[-mid[1][0]/det,mid[0][0]/det]]
            k=[]; norm=0
            for i in range(2):
                center=.5-sum(inv[i][l]*f[l] for l in range(2)); rad=0
                for m in range(2):
                    a=add(scale(j[0][m],-inv[i][0]),scale(j[1][m],-inv[i][1]))
                    a=add(a,(int(i==m),int(i==m)))
                    rad+=max(abs(x) for x in a)
                norm=max(norm,rad); k.append((center-.5*rad-MARGIN,center+.5*rad+MARGIN))
            if any(a>1 or b<0 for a,b in k):
                excluded+=1; k_exclusions.append({'box':box,'krawczyk_local':k}); continue
            if norm < 1 and all(0<a<b<1 for a,b in k):
                roots.append({'box':box,'krawczyk_local':k,'contraction':norm}); continue
        if depth >= max_depth:
            unresolved.append(box); continue
        axis=depth%2
        if axis==0:
            halves=[list(map(list,zip(*part))) for part in zip(*(split(list(col)) for col in zip(*n)))]
        else:
            halves=[list(part) for part in zip(*(split(row) for row in n))]
        bounds=list(box); idx=axis*2; m=(bounds[idx]+bounds[idx+1])/2
        b0=bounds.copy();b0[idx+1]=m;b1=bounds.copy();b1[idx]=m
        stack.extend([(halves[0],b0,depth+1),(halves[1],b1,depth+1)])
    return {'roots':roots,'unresolved':unresolved,'excluded':excluded,'visited':visited,
            'complete_at_station':not unresolved,'hull_exclusions':leaves,'krawczyk_exclusions':k_exclusions}


def station_enclosure(data, guide, distance):
    """Exact-rational Sturm isolation of the two necessary material-hull boundaries."""
    from fractions import Fraction as F
    from math import comb
    n=len(guide)-1
    c=[sum(F(guide[i][0])*comb(n,i)*comb(n-i,k-i)*(-1)**(k-i)
           for i in range(k+1)) for k in range(n+1)]
    points=[]
    for face in data['faces']:
        points.extend(p for row in face.get('poles',[]) for p in row)
        points.extend(face.get('base_poles',[]))
        if 'direction' in face and face['direction'][0] != 0:
            raise ValueError('X hull needs extrusion endpoints')
    bounds=[min(F(p[0]) for p in points)-F(distance),max(F(p[0]) for p in points)+F(distance)]
    def val(p,x):
        y=F(0)
        for a in reversed(p): y=y*x+a
        return y
    def trim(p):
        while p and p[-1]==0: p.pop()
        return p
    def rem(a,b):
        a=a.copy()
        while len(a)>=len(b):
            z=a[-1]/b[-1]; k=len(a)-len(b)
            for i,v in enumerate(b): a[i+k]-=z*v
            trim(a)
        return a
    output=[]
    for bound in bounds:
        p=c.copy();p[0]-=bound;trim(p)
        chain=[p,[i*p[i] for i in range(1,len(p))]]
        while len(chain[-1])>1:
            r=trim([-v for v in rem(chain[-2],chain[-1])])
            if not r: break
            chain.append(r)
        def variations(x):
            signs=[v>0 for q in chain if (v:=val(q,x))]
            return sum(a!=b for a,b in zip(signs,signs[1:]))
        radius=1+max(abs(v/p[-1]) for v in p[:-1])
        stack=[(-radius,radius)];roots=[]
        while stack:
            a,b=stack.pop();count=variations(a)-variations(b)
            if not count: continue
            if count==1 and b-a<F(1,10**10):
                roots.append([a,b]);continue
            m=(a+b)/2;stack.extend([(a,m),(m,b)])
        output.append({'hull_boundary':str(bound),'roots':[[str(a),str(b)] for a,b in sorted(roots)],
                       'approx':[[float(a),float(b)] for a,b in sorted(roots)]})
    return output


def run(source, stations):
    raw=Path(source).read_bytes(); data=json.loads(raw)
    wall=data['faces'][1]
    guide=[row[0] for row in data['faces'][5]['poles']]
    assert len(guide) == len(wall['base_poles'])
    results=[]
    for t in stations:
        g=value(guide,t); tangent=value(derivative(guide),t)
        for i in (1,5,2):
            charts=[isolate(n,b,g,tangent,.008) for b,n in patches(data['faces'][i])]
            results.append({'station':t,'face_index':i,'charts':charts,
                            'complete_at_station':all(x['complete_at_station'] for x in charts),
                            'root_count':sum(len(x['roots']) for x in charts)})
    # Compare the same cap basis with its original trim and full basis rectangle.
    cap = dict(data['faces'][5]); cap['bounds'] = list(cap['bounds'])
    cap['bounds'][0] = cap['uknots'][0]
    untrimmed = [isolate(n,b,value(guide,.1),value(derivative(guide),.1),.008)
                 for b,n in patches(cap)]
    return {'source':str(Path(source).resolve()),'sha256':hashlib.sha256(raw).hexdigest(),
            'distance':.008,'fp_margin':MARGIN,'results':results,
            'station_boundaries':station_enclosure(data,guide,.008),
            'cap_basis_at_point_one':untrimmed,
            'cap_trim_u_min':data['faces'][5]['bounds'][0],
            'limits':'Fixed stations only; no continuous branch, material trim, terminal ordering or production admission certificate.'}


if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('source');p.add_argument('--stations',type=float,nargs='+',default=[.1,.5,.9,1]);p.add_argument('--output',required=True)
    a=p.parse_args();out=run(a.source,a.stations);Path(a.output).write_text(json.dumps(out,indent=2))
    for x in out['results']:print(x['station'],x['face_index'],x['root_count'],x['complete_at_station'],sum(c['visited'] for c in x['charts']))
