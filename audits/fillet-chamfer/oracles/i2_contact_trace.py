"""CS/CO physical contact controller and candidate-chart adapter evidence.

Source hashes bind conventions, not executed Rust coefficients. The current
candidate constructs a different canal net and un-enclosed three-point circles.
Those identities cannot be inferred from the independent oracle controls.
"""
import argparse
import hashlib
import json
import math
from pathlib import Path
from i2_trim_bounds import I, add, const, inv, mul, scale, sqrt, implicit, coefficients, up
from i2_canal_controls import scalars
from i2_endpoint_trace import midpoint, error, basis_error, inverse_cos
from i2_paired_trace import PI, angle, lipschitz, support_arithmetic


def jets(kind,t,r,c):
    a,b,n,m=c
    variable=const(t); variable[1]=I(1)
    h,alpha,beta,_=scalars(variable,r,c)
    if kind=='CS':
        p=[h,const(0),variable]
        return p,{'C':[variable,const(0)],'S':[add(scale(variable,-10),const(14.644660940673)),add(scale(h,10),const(-250))]}
    q=add(const(b-r*(1+m*m).sqrt()),scale(variable,-m))
    cosine=scale(inv(q),-I(r)/(1+m*m).sqrt())
    sine=sqrt(add(const(1),scale(mul(cosine,cosine),-1)))
    p=[add(h,scale(mul(alpha,sine),r)),scale(add(const(1),scale(cosine,-1)),r),add(variable,scale(mul(beta,sine),r))]
    theta=scale(angle(p[0],p[1]),-1)
    eta=scale(add(scale(p[2],-1),const(-I(-50.000001)/10)),I(10)/(I(1.000001)-I(-50.000001)))
    return p,{'C':[variable,mul(sine,inv(add(const(1),cosine)))],'O':[theta,eta]}


def construct(kind,r,c,canal,shared=None):
    a,b,n,m=c
    base=-r*m/(1+m*m).sqrt()
    ends=[I(r),implicit(I(0),r,c,'CI')[1][0]] if kind=='CS' else [base,(b-a)/(m+n)+base]
    lo,hi=ends[0].a,ends[1].b
    fourth={}; first={}
    for i in range(256):
        p,uv=jets(kind,I(lo+(hi-lo)*i/256,lo+(hi-lo)*(i+1)/256),r,c)
        for key,vec in {'spatial':p,**uv}.items():
            fourth[key]=max(fourth.get(key,0.),up(sum(v[4].norm()*24 for v in vec)))
            first[key]=max(first.get(key,0.),up(sum(v[1].norm() for v in vec)))
    # Choose against each disjoint derivative and trace allocation, with finite cap.
    count=max(1,math.ceil((hi-lo)/min(.01,(1e-11*384/max(fourth.values()))**.25,(1e-8/max(fourth.values()))**(1/3))))
    if count>16384: raise ValueError('contact controller cap')
    knots=[midpoint(ends[0])+(midpoint(ends[1])-midpoint(ends[0]))*i/count for i in range(count+1)]
    result={}
    for i,(u,v) in enumerate(zip(knots,knots[1:])):
        p0,uv0=jets(kind,I(u),r,c); p1,uv1=jets(kind,I(v),r,c)
        for key,left in {'spatial':p0,**uv0}.items():
            right={'spatial':p1,**uv1}[key]; dt=I(v)-I(u)
            boxes=[[x[0] for x in left],[x[0]+dt*x[1]/3 for x in left],
                   [x[0]-dt*x[1]/3 for x in right],[x[0] for x in right]]
            points=[[midpoint(x) for x in box] for box in boxes]
            row=result.setdefault(key,{'controls':[],'control_error':0.})
            row['controls'].append(points)
            row['control_error']=max(row['control_error'],*(error(x,y) for x,y in zip(points,boxes)))
    h=up(max(v-u for u,v in zip(knots,knots[1:])))
    for key,row in result.items():
        row['endpoint_charge']=0.
        for end,t in enumerate(ends):
            p,uv=jets(kind,t,r,c); box=[v[0] for v in {'spatial':p,**uv}[key]]
            point=[midpoint(v) for v in box]
            old=row['controls'][0 if end==0 else -1][0 if end==0 else -1]
            row['endpoint_charge']=max(row['endpoint_charge'],up(error(point,box)+sum(abs(x-y) for x,y in zip(old,point))+(t.b-t.a)*first[key]))
            row['controls'][0 if end==0 else -1][0 if end==0 else -1]=point
        row['fourth_L1']=fourth[key]; row['derivative_remainder']=up(fourth[key]*h**3)
        row['truncation']=up(fourth[key]*h**4/384)
        row['evaluation']=basis_error(max(sum(abs(x) for x in p) for span in row['controls'] for p in span))
    if shared:
        for key,points in shared.items():
            row=result[key]
            for end,point in enumerate(points):
                old=row['controls'][0 if end==0 else -1][0 if end==0 else -1]
                row['endpoint_charge']=up(row['endpoint_charge']+sum(abs(x-y) for x,y in zip(point,old)))
                row['controls'][0 if end==0 else -1][0 if end==0 else -1]=point
    spatial=result['spatial']
    for s,row in result.items():
        if s=='spatial': continue
        L=max(lipschitz('I' if s=='O' else s,r))
        ar=support_arithmetic('I' if s=='O' else s,r,canal)
        row['known_sum']=up(sum(spatial[k]+L*row[k] for k in ['truncation','evaluation','control_error','endpoint_charge'])+sum(ar[k] for k in ['basic','construction','fit']))
        row['runtime_trig_factor']=ar['trig_factor']
    return {'kind':kind,'radius':r,'knots':knots,'physical_domain_boxes':[[x.a,x.b] for x in ends],
            'traces':result,'full_admission':False,'candidate_endpoint_identity_proven':False}


def adapt(s,point,b,contact):
    if s=='C': return [point[1],point[0]]
    if s=='A': return [point[1]-midpoint(PI)/2,point[0]+2*midpoint(PI)-contact]
    if s=='B': return [point[0],point[1]/b]
    return list(point)


def candidate_adapters(paired,b,contact):
    result={}
    for kind,rows in paired['paired_traces'].items():
        result[kind]={}
        for s,row in rows.items():
            controls=[[adapt(s,p,midpoint(b),midpoint(contact)) for p in span] for span in row['controls']]
            # gamma(16) covers b division and the two angular shifts in stored UV.
            conversion=up(16*2**-53*max(sum(abs(x) for x in p) for span in controls for p in span))
            if s=='A':
                conversion=up(conversion+2.5*(PI.b-PI.a)+(contact.b-contact.a))
            if s=='B':
                xmax=max(abs(p[1]) for span in row['controls'] for p in span)
                conversion=up(conversion+xmax*(b.b-b.a)/(b.a*b.a))
            L=lipschitz(s,paired['radius'])
            transformed_L=[L[1],L[0]] if s in ['A','C'] else [L[0],L[1]*b.b] if s=='B' else L
            result[kind][s]={'controls':controls,'conversion_error':conversion,
                'composed_conversion_error':up(max(transformed_L)*conversion),
                'candidate_lipschitz':transformed_L,
                'derivative_adapter':{'A':'swap; shifts have zero derivative','C':'swap','B':'(uprime,vprime/b)'}.get(s,'identity'),
                'same_candidate_surface_coefficients_proven':False}
    return result


def main():
    p=argparse.ArgumentParser(); p.add_argument('--evidence',type=Path,required=True); args=p.parse_args()
    c=coefficients(Path('target/i2-representation-budget-20261104/results.json'))
    source=Path('truck-shapeops/src/fillet/i2.rs')
    source_bytes=source.read_bytes()
    data={'candidate_source_sha256':hashlib.sha256(source_bytes).hexdigest(),
        'circle_builder_sha256':hashlib.sha256(Path('truck-modeling/src/geom_impls.rs').read_bytes()).hexdigest(),
        'full_admission':False,'radii':{}}
    snapshot=args.evidence/('candidate-'+data['candidate_source_sha256']+'.rs.snapshot')
    if not snapshot.exists(): snapshot.write_bytes(source_bytes)
    for label,r in [('r05',.5),('r1-final',1.),('r15',1.5)]:
        pairpath=args.evidence/(label+'.json')
        paired=json.loads(pairpath.read_text())
        canalpath=Path('target/i2-endpoint-trace-20261105')/('canal-'+label.replace('-final','')+'-final.json')
        canal=json.loads(canalpath.read_text())
        contact=inverse_cos(-I(1)/(1+c[2]*c[2]).sqrt())
        physical=json.loads((Path('target/i2-endpoint-trace-20261105')/(label.replace('-final','')+'.json')).read_text())
        shared={}
        for kind,ends in {'CS':[('BC',0),('CI',0)],'CO':[('DC',1),('CI',1)]}.items():
            shared[kind]={'spatial':[physical['curves'][k]['controls'][0 if e==0 else -1][0 if e==0 else -1] for k,e in ends],
                          'C':[paired['paired_traces'][k]['C']['controls'][0 if e==0 else -1][0 if e==0 else -1] for k,e in ends]}
        data['radii'][str(r)]={'oracle_paired_sha256':hashlib.sha256(pairpath.read_bytes()).hexdigest(),
            'oracle_canal_sha256':hashlib.sha256(canalpath.read_bytes()).hexdigest(),
            'adapters':candidate_adapters(paired,c[1],contact),
            'contacts':{k:construct(k,r,c,canal,shared[k]) for k in ['CS','CO']}}
    data['unmet']=['Candidate canal uses 256 t spans and s_end=tan(phi_ridge/2); oracle uses 130 spans and s_end=1.1: coefficient identity is false.',
        'Candidate A/B circle_arc circumcenter and axis controls are not serialized; analytic phase adapters do not prove those coefficients.',
        'Candidate fitted uses 1024 spans and sign-reversed CI/BC/AB/CS domains; bind controls, endpoint corrections and arithmetic before admission.',
        'Runtime sin_cos absolute-error bound remains unspecified.']
    with (args.evidence/'candidate-adapters-contacts-shared.json').open('x') as f: json.dump(data,f,indent=2)
    print(json.dumps({r:{k:{s:v.get('known_sum') for s,v in row['traces'].items() if s!='spatial'} for k,row in v['contacts'].items()} for r,v in data['radii'].items()},indent=2))


if __name__=='__main__': main()
