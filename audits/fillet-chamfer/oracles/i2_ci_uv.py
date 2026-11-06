"""Stored CI UV cubics in Truck's (eta,theta) and canal (T,tan(phi/2)) charts.

Independent spatial and UV controls share precisely the physical knot vector.
The cone chart is the retained imported generatrix, in normalized coordinates.
This does not assert a bound on Truck's support/processor evaluation.
"""
import argparse
import json
from pathlib import Path
from i2_trim_bounds import I, add, coefficients, const, implicit, inv, mul, scale, sincos, up
from i2_endpoint_trace import basis_error, error, midpoint


def atan_positive(x):
    if x.a < 0:
        # Outward arithmetic at Y=0 may introduce a subnormal negative endpoint.
        if x.a < -1e-12:
            raise ValueError('atan branch')
    if x.norm() >= .2:
        raise ValueError('atan series domain')
    term,result = x,x
    for k in range(1,20):
        term = term*(-x*x)
        result = result+term/(2*k+1)
    remainder = up(x.norm()**41/41)
    return result+I(-remainder,remainder)


def charts(phi,r,coeff):
    p,t,_ = implicit(phi,r,coeff,'CI')
    x,y,z = p
    dx = [x[j+1]*(j+1) for j in range(4)]+[I(0)]
    dy = [y[j+1]*(j+1) for j in range(4)]+[I(0)]
    angular_der = mul(add(mul(x,dy),scale(mul(y,dx),-1)),inv(add(mul(x,x),mul(y,y))))
    theta = [atan_positive(y[0]/x[0])]+[angular_der[j-1]/j for j in range(1,5)]
    # Actual binary64 imported line endpoints, each divided by ten, and
    # processor translation +50/10. Arithmetic stays interval-enclosed.
    z0 = I(-50.000001000000005)/10
    z1 = I(1.000001)/10
    eta = scale(add(z,const(-5-z0)),I(1)/(z1-z0))
    s,c = sincos(phi)
    tangent = mul(s,inv(add(const(1),c)))
    return {'inner_cone_eta_theta':[eta,theta], 'canal_T_tan_half_phi':[t,tangent]}


def construct(coeff,physical,previous):
    r = physical['radius']
    spatial = physical['curves']['CI']
    knots = spatial['knots']
    bounds = previous['radii'][str(r)]['CI']
    result = {}
    values = charts(I(knots[0]),r,coeff)
    for name in values:
        result[name] = {'controls':[],'control_error':0.,'max_control':0.}
    for a,b in zip(knots,knots[1:]):
        current = charts(I(b),r,coeff)
        for name,row in result.items():
            before,after = values[name],current[name]
            h = I(b)-I(a)
            boxes = [[v[0] for v in before], [v[0]+v[1]*h/3 for v in before],
                     [v[0]-v[1]*h/3 for v in after], [v[0] for v in after]]
            controls = [[midpoint(v) for v in box] for box in boxes]
            row['control_error'] = max(row['control_error'],*(error(p,q) for p,q in zip(controls,boxes)))
            row['max_control'] = max(row['max_control'],*(sum(abs(v) for v in p) for p in controls))
            row['controls'].append(controls)
        values = current
    h = up(max(b-a for a,b in zip(knots,knots[1:])))
    for name,row in result.items():
        row['evaluation_error'] = basis_error(row['max_control'])
        if name.startswith('inner'):
            fourth = bounds['inner_cone_chart_fourth_bounds'][::-1]
            lipschitz = [13.,43.]
        else:
            fourth = bounds['canal_uv_fourth_bounds_t_tan_half_phi']
            lipschitz = [2.5,5*r]
        row['truncation_composed'] = up(sum(a*b for a,b in zip(fourth,lipschitz))*h**4/384)
        row['stored_and_basis_composed'] = up(max(lipschitz)*(row['control_error']+row['evaluation_error']))
        row['spatial_truncation'] = up(bounds['derivative_L1_bounds'][3]*h**4/384)
        row['known_same_parameter_terms'] = up(row['truncation_composed']+row['stored_and_basis_composed']+
            row['spatial_truncation']+spatial['control_enclosure_L1']+
            spatial['truck_cubic_basis_evaluation_L1']+spatial['physical_endpoint_charge'])
        row['remaining_of_1e_9_for_surface_and_chart_endpoint'] = 1e-9-row['known_same_parameter_terms']
        row['full_admission'] = False
        row['unmet'] = 'Actual stored support controls, surface/processor evaluation and physical UV endpoint corrections.'
        row['knots'] = knots
    return result


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--coefficients',type=Path,required=True)
    p.add_argument('--physical',type=Path,required=True)
    p.add_argument('--previous',type=Path,required=True)
    p.add_argument('--output',type=Path,required=True)
    a = p.parse_args()
    result = construct(coefficients(a.coefficients),json.loads(a.physical.read_text()),json.loads(a.previous.read_text()))
    a.output.write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps({k:{key:value for key,value in v.items() if key not in ['controls','knots']} for k,v in result.items()},indent=2))


if __name__ == '__main__':
    main()
