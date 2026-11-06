"""Paired physical I2 UV controls; explicit L1 composition and evaluator ledger.

The support arithmetic bound separates basic operations from the runtime sin_cos
contract. A missing trigonometric error bound is rejection, never zero error.
All serialized lengths/bounds are normalized; original plane UVs are capture units.
"""
import argparse
import json
import math
from pathlib import Path
from i2_trim_bounds import I, add, const, implicit, inner, inv, mul, scale, sincos, coefficients, up
from i2_ci_uv import atan_positive, charts as ci_charts
from i2_endpoint_trace import midpoint, error, basis_error

U = 2**-53
PI = I(math.nextafter(math.pi, -math.inf), math.nextafter(math.pi, math.inf))
BASE = [[-23.22330470336,176.77669529663,0.],
        [-23.930411484546546,177.48380207781656,0.],
        [-22.516197922173454,177.48380207781656,0.]]


def angle(x,y):
    dx = [x[j+1]*(j+1) for j in range(4)]+[I(0)]
    dy = [y[j+1]*(j+1) for j in range(4)]+[I(0)]
    d = mul(add(mul(x,dy),scale(mul(y,dx),-1)),inv(add(mul(x,x),mul(y,y))))
    return [atan_positive(y[0]/x[0])]+[d[j-1]/j for j in range(1,5)]


def charts(kind,phi,r,coeff):
    t = None
    if kind in ['CI','BC','DC']:
        p,t,_ = implicit(phi,r,coeff,kind)
    else:
        p = inner(phi,r,coeff,kind)
    x,y,z = p
    variable = const(phi); variable[1] = I(1)
    if kind == 'CI':
        c = ci_charts(phi,r,coeff)
        return {'I':c['inner_cone_eta_theta'],'C':c['canal_T_tan_half_phi']}
    result = {}
    if t is not None:
        s,c = sincos(phi)
        result['C'] = [t,mul(s,inv(add(const(1),c)))]
    if kind in ['BC','AB']:
        result['B'] = [variable,x]
    if kind == 'DC':
        o,a,b = [[I(v) for v in row] for row in BASE]
        ax,ay = a[0]-o[0],a[1]-o[1]
        bx,by = b[0]-o[0],b[1]-o[1]
        xx = add(scale(x,10),const(-200-o[0]))
        yy = add(scale(y,10),const(-o[1]))
        det = ax*by-ay*bx
        result['D'] = [scale(add(scale(xx,by),scale(yy,-bx)),I(1)/det),
                       scale(add(scale(yy,ax),scale(xx,-ay)),I(1)/det)]
    if kind in ['AB','AS']:
        # Continuous meridian across AB(0)==AS(pi), avoiding a 2*pi UV jump.
        meridian = add(variable,const(-PI if kind == 'AB' else -2*PI))
        result['A'] = [meridian,angle(x,y)]
    if kind == 'AS':
        result['S'] = [add(scale(z,-10),const(I(14.644660940673))),
                       add(scale(x,10),const(-250))]
    return result


def lipschitz(support,r):
    # Coordinatewise L1 derivative bounds on the whole support neighborhood.
    # C: |Pt|_1<2.5, |Ps|_1<=2r(sqrt(2)+1)<5r for s>=0.
    # I: line dx,dz=5.1000002, rho<=30; rotation adds sqrt(2).
    return {'C':[2.5,5*r], 'I':[13.,43.], 'B':[2*r,1.],
            'A':[3*r,45.], 'D':[.142,.142], 'S':[.1,.1]}[support]


def support_arithmetic(support,r,canal):
    if support == 'C':
        return {'basic':canal['projected_evaluation_bound'],
                'construction':canal['projected_control_enclosure'],
                'fit':canal['fit_truncation'],'trig_factor':0.}
    # gamma(512) encloses the <=128 basic operations in Line, unit-Z rotation,
    # affine Processor / Extruded, and Plane subs, including cancellation.
    # Bounds use absolute intermediates (not final coordinates): <=100 for I,
    # <=80 for A/B, <=100 for planes after normalization. Axis is exactly +Z.
    # Similarities are charged separately; arbitrary dense processors excluded.
    magnitude = 100. if support in ['I','D','S'] else 80.
    basic = up((512*U/(1-512*U))*magnitude)
    # Per sin or cos absolute error e. I: two radial terms <=2*30e.
    # A: meridian plus angular errors <=(4r+2R)e+2r*e², padded for e<=1e-6.
    factor = {'I':60.,'A':70.,'B':2*r,'D':0.,'S':0.}[support]
    return {'basic':basic,'construction':up(128*U*magnitude),
            'fit':0.,'trig_factor':factor}


def gate(known,trig_factor,trig_error=None,scale_factor=1.,translation=0.):
    if not all(math.isfinite(x) for x in [known,trig_factor,scale_factor,translation]) or scale_factor<=0:
        return False
    if trig_factor and (trig_error is None or not math.isfinite(trig_error) or not 0<=trig_error<=1e-6):
        return False
    frame = similarity_charge(scale_factor,translation)
    return frame <= 1e-9*scale_factor and scale_factor*(known+trig_factor*(trig_error or 0))+2*frame <= 1e-9*scale_factor


def similarity_charge(scale_factor,translation):
    # Normalize capture's 300 extent to 30. Charge both compared traces.
    return up(256*U*(abs(translation)+30*scale_factor))


def construct(coeff,physical,canal):
    r = physical['radius']
    if r not in [.5,1.,1.5] or any(v.norm()>limit for v,limit in zip(coeff,[20.001,30.001,1.001,1.001])) or any(v.a<limit for v,limit in zip(coeff,[19.999,29.999,.999,.999])):
        raise ValueError('outside established coefficient/radius family')
    result = {}
    for kind,spatial in physical['curves'].items():
        knots = spatial['knots']
        lo,hi = spatial['physical_domain_boxes'][0][0],spatial['physical_domain_boxes'][1][1]
        sample = charts(kind,I(knots[0]),r,coeff)
        rows = {s:{'controls':[],'control_error':0.,'fourth':[0.,0.],'first':[0.,0.]} for s in sample}
        count = math.ceil((hi-lo)/.03)
        for i in range(count):
            values = charts(kind,I(lo+(hi-lo)*i/count,lo+(hi-lo)*(i+1)/count),r,coeff)
            for s,uv in values.items():
                for j in range(2):
                    rows[s]['fourth'][j] = max(rows[s]['fourth'][j],up(24*uv[j][4].norm()))
                    rows[s]['first'][j] = max(rows[s]['first'][j],uv[j][1].norm())
        previous = sample
        for a,b in zip(knots,knots[1:]):
            current = charts(kind,I(b),r,coeff)
            for s,row in rows.items():
                l,rr = previous[s],current[s]
                h = I(b)-I(a)
                boxes = [[v[0] for v in l],[v[0]+h*v[1]/3 for v in l],
                         [v[0]-h*v[1]/3 for v in rr],[v[0] for v in rr]]
                controls = [[midpoint(v) for v in box] for box in boxes]
                row['control_error'] = max(row['control_error'],*(error(p,q) for p,q in zip(controls,boxes)))
                row['controls'].append(controls)
            previous = current
        for s,row in rows.items():
            row['endpoint_charge'] = 0.
            for end,box in enumerate(spatial['physical_domain_boxes']):
                uv = charts(kind,I(*box),r,coeff)[s]
                point = [midpoint(v[0]) for v in uv]
                old = row['controls'][0][0] if end==0 else row['controls'][-1][-1]
                charge = error(point,[v[0] for v in uv])+sum(abs(a-b) for a,b in zip(point,old))
                charge += (box[1]-box[0])*sum(row['first'])
                row['endpoint_charge'] = max(row['endpoint_charge'],up(charge))
                row['controls'][0 if end==0 else -1][0 if end==0 else -1] = point
            row['knots'] = knots
        result[kind] = rows
    # The same support shares identical UV endpoint arrays at common vertices.
    for k,j,s in [('BC','DC','C'),('AB','AS','A')]:
        ei,ej = (1,0) if k=='BC' else (0,1)
        a=result[k][s]['controls'][-1 if ei else 0][-1 if ei else 0]
        b=result[j][s]['controls'][-1 if ej else 0][-1 if ej else 0]
        point=[(x+y)/2 for x,y in zip(a,b)]
        for kind,end,old in [(k,ei,a),(j,ej,b)]:
            row=result[kind][s]
            row['endpoint_charge']=up(row['endpoint_charge']+sum(abs(x-y) for x,y in zip(point,old)))
            row['controls'][-1 if end else 0][-1 if end else 0]=point
    for kind,rows in result.items():
        spatial=physical['curves'][kind]
        knots=spatial['knots']
        h=up(max(b-a for a,b in zip(knots,knots[1:])))
        # Independently bound spatial fourth derivatives on the same complete domain.
        m4=0.
        lo,hi=spatial['physical_domain_boxes'][0][0],spatial['physical_domain_boxes'][1][1]
        count=math.ceil((hi-lo)/.03)
        for i in range(count):
            phi=I(lo+(hi-lo)*i/count,lo+(hi-lo)*(i+1)/count)
            p=implicit(phi,r,coeff,kind)[0] if kind in ['CI','BC','DC'] else inner(phi,r,coeff,kind)
            m4=max(m4,up(sum(v[4].norm()*24 for v in p)))
        for s,row in rows.items():
            lip=lipschitz(s,r)
            arithmetic=support_arithmetic(s,r,canal)
            maximum=max(sum(abs(x) for x in p) for span in row['controls'] for p in span)
            limits={'C':[[-1.5,5.],[0.,1.1]],'I':[[0.,1.],[-1e-12,.2]],
                    'A':[[-4.,0.],[-1e-12,.2]],'B':[[-1e-12,1.571],[0.,31.]],
                    'D':[[-500.,500.],[-500.,500.]],'S':[[-100.,100.],[-300.,100.]]}[s]
            if any(not limits[j][0]<=p[j]<=limits[j][1] for span in row['controls'] for p in span for j in range(2)):
                raise ValueError('UV convex hull outside bounded support neighborhood')
            row['whole_support_uv_neighborhood']=limits
            row['ledger']={
                'spatial_truncation':up(m4*h**4/384),
                'spatial_controls':spatial['control_enclosure_L1'],
                'spatial_evaluation':spatial['truck_cubic_basis_evaluation_L1'],
                'spatial_endpoint':spatial['physical_endpoint_charge'],
                'uv_truncation':up(sum(a*b for a,b in zip(lip,row['fourth']))*h**4/384),
                'uv_controls':up(max(lip)*row['control_error']),
                'uv_evaluation':up(max(lip)*basis_error(maximum)),
                'uv_endpoint':up(max(lip)*row['endpoint_charge']),
                'support_controls':arithmetic['construction'],
                'support_evaluation_basic':arithmetic['basic'],
                'support_fit':arithmetic['fit']}
            known=up(sum(row['ledger'].values()))
            row['known_sum']=known
            row['runtime_trig_factor']=arithmetic['trig_factor']
            row['required_runtime_trig_abs_error_max']=(1e-9-known)/arithmetic['trig_factor'] if arithmetic['trig_factor'] else None
            row['trace_proven_without_runtime_trig_contract']=gate(known,arithmetic['trig_factor'])
            row['full_admission']=False
    R=coeff[0]+coeff[2]*r+r*(1+coeff[2]*coeff[2]).sqrt()
    return {'radius':r,'support_definitions':{
            'A':{'R':midpoint(R),'R_box':[R.a,R.b],
                 'equation':'((R+r*cos(u))*cos(v),(R+r*cos(u))*sin(v),r+r*sin(u))',
                 'chart_order':['meridian','azimuth']},
            'B':{'equation':'(v,r*(1-cos(u)),r*(1-sin(u)))','chart_order':['phi','X']},
            'I':{'line_capture':[[199.99999900004548,0.,-50.000001000000005],[251.0000009999991,0.,1.000001]],
                 'processor_translation_capture':[-200.,0.,50.],'orientation':True,'chart_order':['eta','theta']},
            'D':{'plane_capture':BASE},
            'S':{'plane_capture':[[50.,0.,14.644660940673],[50.,0.,13.644660940673],[51.,0.,14.644660940673]]},
            'C':{'equation':'Use existing degree-(3,2) net without reconstruction','chart_order':['T','tan(phi/2)']}},
            'paired_traces':result,'full_admission':False,
            'runtime_trig_abs_error':None,
            'remaining':'Bound runtime sin_cos over complete angular support domains; Rust builder/QA separate.'}


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--coefficients',type=Path,required=True)
    parser.add_argument('--physical',type=Path,required=True)
    parser.add_argument('--canal',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    a=parser.parse_args()
    result=construct(coefficients(a.coefficients),json.loads(a.physical.read_text()),json.loads(a.canal.read_text()))
    with a.output.open('x') as f: json.dump(result,f,indent=2); f.write('\n')
    print(json.dumps({k:{s:{'known_sum':v['known_sum'],'trig_factor':v['runtime_trig_factor'],
        'required_trig_bound':v['required_runtime_trig_abs_error_max']} for s,v in row.items()}
        for k,row in result['paired_traces'].items()},indent=2))


if __name__=='__main__':
    main()
