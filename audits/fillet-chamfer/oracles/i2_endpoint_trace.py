"""Physical I2 trim controls and restricted Truck cubic basis error bounds.

Consumes previous derivative evidence without regenerating it. This is executable
candidate construction, not admission: support compositions are separate obligations.
All lengths are normalized as in i2_trim_bounds. No tolerance-sized merging.
"""
import argparse
import copy
import json
import math
from pathlib import Path
from i2_trim_bounds import I, coefficients, implicit, inner, stored_controls, trig, up


def midpoint(x):
    return (x.a+x.b)/2


def inverse_cos(c):
    if c.a < -1 or c.b > 1:
        raise ValueError('cosine outside invertible branch')
    # cos is strictly decreasing here. Separate searches preserve uncertainty
    # in the target and in the outward-rounded trigonometric evaluator.
    ends = []
    for lower in [True, False]:
        lo, hi = 0., 3.142
        for _ in range(52):
            mid = (lo+hi)/2
            v = trig(I(mid), True)
            if (v.a > c.b if lower else v.b >= c.a):
                lo = mid
            else:
                hi = mid
        ends.append(lo if lower else hi)
    return I(*ends)


def domains(coeff, r):
    a,b,n,m = coeff
    L = (1+m*m).sqrt()
    base_t = -r*m/L
    ridge_t = (b-a)/(m+n)+base_t
    def outer(t):
        return inverse_cos(-I(r)/(L*(b-m*t-r*L)))
    half_pi = inverse_cos(I(0))
    pi = I(math.nextafter(math.pi, -math.inf), math.nextafter(math.pi, math.inf))
    inner_contact = inverse_cos(-I(1)/(1+n*n).sqrt())
    return {'CI': (I(0),outer(ridge_t)), 'BC': (I(0),half_pi),
            'DC': (half_pi,outer(base_t)), 'AB': (I(0),half_pi),
            'AS': (inner_contact,pi)}


def endpoint(kind, phi, r, coeff):
    if kind in ['CI','BC','DC']:
        return [v[0] for v in implicit(phi,r,coeff,kind)[0]]
    return [v[0] for v in inner(phi,r,coeff,kind)]


def hull(points):
    return [I(min(p[j].a for p in points),max(p[j].b for p in points)) for j in range(3)]


def error(point, box):
    return up(sum(up(max(abs(v-x.a),abs(v-x.b))) for v,x in zip(point,box)))


def basis_error(max_control, degree=3):
    if degree != 3 or not math.isfinite(max_control) or max_control < 0:
        raise ValueError('only finite nonnegative cubic control hulls supported')
    # Restricted to cubic Bezier spans: clamped ends x4, internal knots x3.
    # Every nonzero recurrence denominator equals a positive local span h.
    # For normal arithmetic, reciprocal/subtract/multiply error in b <= 6u;
    # a=1-b has absolute error <=8u, including cancellation. Exact basis
    # entries and factors are in [0,1]. Conservatively E_k<=2E_(k-1)+20u
    # (E_0=0), hence E_3<=140u per entry. Four control multiplications and
    # sequential additions then cost <600u*M in L1. 2048u*M also covers
    # second-order terms. This does NOT apply to arbitrary knot multiplicity,
    # extrapolation, rational projection, or surface/processor evaluation.
    return up(2048*2**-53*max_control)


def construct(coeff, r, previous):
    physical = domains(coeff,r)
    rows = copy.deepcopy(previous['radii'][str(r)])
    boxes = {}
    for kind,(a,b) in physical.items():
        old = rows[kind]
        lo,hi = midpoint(a),midpoint(b)
        if not old['domain'][0] <= a.a <= b.b <= old['domain'][1]:
            raise ValueError('physical interval outside established derivative domain')
        max_step = (old['domain'][1]-old['domain'][0])/old['spans']
        old['domain'] = [lo,hi]
        old['spans'] = math.ceil((hi-lo)/max_step)
        boxes[kind] = [endpoint(kind,a,r,coeff),endpoint(kind,b,r,coeff)]
    curves = stored_controls(coeff,r,rows)
    # Each joint gets one point, referenced by both incident nonlinear edges.
    groups = [[('BC',1),('DC',0)],[('AB',0),('AS',1)]]
    covered = {x for group in groups for x in group}
    groups += [[(kind,end)] for kind in curves for end in [0,1] if (kind,end) not in covered]
    vertices = []
    endpoint_error = 0.
    for group in groups:
        box = hull([boxes[k][i] for k,i in group])
        point = [midpoint(v) for v in box]
        enclosure = error(point,box)
        endpoint_error = max(endpoint_error,enclosure)
        correction = 0.
        for kind,end in group:
            controls = curves[kind]['controls']
            old = controls[0][0] if end == 0 else controls[-1][-1]
            correction = max(correction,up(sum(abs(a-b) for a,b in zip(old,point))))
            if end == 0:
                controls[0][0] = point
            else:
                controls[-1][-1] = point
            curves[kind]['physical_endpoint_charge'] = max(curves[kind].get('physical_endpoint_charge',0.),enclosure+correction)
        vertices.append({'uses':group,'point':point,'physical_box':[[v.a,v.b] for v in box],
                         'enclosure_L1':enclosure,'control_correction_L1':correction})
    for kind,c in curves.items():
        # The stored knots are the physical midpoint approximations; the tiny
        # omitted/added endpoint slivers are explicitly charged using M1.
        a,b = physical[kind]
        sliver = max(a.b-a.a,b.b-b.a)*rows[kind]['derivative_L1_bounds'][0]
        c['physical_endpoint_charge'] = up(c['physical_endpoint_charge']+sliver)
        if min(y-x for x,y in zip(c['knots'],c['knots'][1:])) < 1e-100:
            raise ValueError('unsupported arithmetic range')
        c['max_control_L1'] = max(sum(abs(v) for v in p) for span in c['controls'] for p in span)
        c['truck_cubic_basis_evaluation_L1'] = basis_error(c['max_control_L1'])
        c['physical_domain_boxes'] = [[a.a,a.b],[b.a,b.b]]
        c['bspline_knots'] = [c['knots'][0]]*4 + [k for k in c['knots'][1:-1] for _ in range(3)] + [c['knots'][-1]]*4
        c['bspline_controls'] = c['controls'][0]+[p for span in c['controls'][1:] for p in span[1:]]
        if max(c['physical_endpoint_charge'],c['truck_cubic_basis_evaluation_L1'],c['control_enclosure_L1']) >= 1e-10:
            raise ValueError('unchanged endpoint/control/evaluation allocation exceeded')
    return {'radius':r,'vertices':vertices,'curves':curves,'full_admission':False,
            'unmet':'Stored paired UV, fitted surface controls and Truck support-composed evaluation remain unbounded.',
            'same_parameter_allocation':1e-9,'endpoint_allocation':1e-10,
            'control_allocation':1e-10,'evaluation_allocation':1e-10}


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--coefficients',type=Path,required=True)
    p.add_argument('--previous',type=Path,required=True)
    p.add_argument('--radius',type=float,choices=[.5,1.,1.5],required=True)
    p.add_argument('--output',type=Path,required=True)
    args = p.parse_args()
    result = construct(coefficients(args.coefficients),args.radius,json.loads(args.previous.read_text()))
    args.output.write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps({k:{'spans':v['span_count'],'domain':v['physical_domain_boxes'],
                         'endpoint':v['physical_endpoint_charge'],'evaluation':v['truck_cubic_basis_evaluation_L1']}
                      for k,v in result['curves'].items()},indent=2))


if __name__ == '__main__':
    main()
