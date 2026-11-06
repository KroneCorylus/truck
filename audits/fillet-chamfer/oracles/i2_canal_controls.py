"""Executable degree-(3,2) positive-weight I2 canal control net.

Uses the global rational parameter s=tan(phi/2), not the angle or the usual
piecewise circular-arc parameter. No new Truck geometry type is needed.
"""
import argparse
import json
import math
from pathlib import Path
from i2_trim_bounds import I, add, coefficients, const, inv, mul, scale, sqrt, up
from i2_endpoint_trace import error, midpoint


def scalars(t,r,coeff):
    _,b,_,m = coeff
    q = add(const(b-r*(1+m*m).sqrt()),scale(t,-m))
    h = sqrt(add(mul(q,q),const(-r*r)))
    d = sqrt(add(scale(mul(q,q),1+m*m),const(-r*r)))
    return [h,mul(h,inv(d)),scale(mul(q,inv(d)),m),t]


def construct(coeff,physical):
    r = physical['radius']
    # Proven extended t support covers the whole physical C patch, whose
    # boundaries remain the separately constructed physical trim curves.
    knots = [-1.5+6.5*i/130 for i in range(131)]
    # A fixed exactly stored s limit covers all physical phi<1.63.
    # Exact quadratic polynomials W=1+s², S=2s, C=1-s² have these
    # Bernstein controls on s in [0,1.1]. Positive weights need no arc split.
    limit = I(1.1)
    W = [I(1),I(1),1+limit*limit]
    S = [I(0),limit,2*limit]
    C = [I(1),I(1),1-limit*limit]
    patches = []
    control_error,weight_error,max_control,fit = 0.,0.,0.,0.
    for a,b in zip(knots,knots[1:]):
        left,right = const(a),const(b)
        left[1]=right[1]=I(1)
        l,rr = scalars(left,r,coeff),scalars(right,r,coeff)
        h = I(b)-I(a)
        hermite = [[v[0] for v in l],[v[0]+h*v[1]/3 for v in l],
                   [v[0]-h*v[1]/3 for v in rr],[v[0] for v in rr]]
        patch = []
        for H,A,B,T in hermite:
            row = []
            for w,s,c in zip(W,S,C):
                box = [H*w+r*A*s,r*(w-c),T*w+r*B*s,w]
                point = [midpoint(v) for v in box]
                control_error = max(control_error,error(point[:3],box[:3]))
                weight_error = max(weight_error,error(point[3:],box[3:]))
                max_control = max(max_control,sum(abs(v) for v in point[:3]))
                row.append(point)
            patch.append(row)
        patches.append(patch)
        variable = const(I(a,b));variable[1]=I(1)
        fourth = [24*v[4].norm() for v in scalars(variable,r,coeff)]
        fit = max(fit,up((fourth[0]+r*(fourth[1]+fourth[2]))*h.b**4/384))
    net = patches[0]+[row for p in patches[1:] for row in p[1:]]
    for p,q in zip(patches,patches[1:]):
        assert p[-1]==q[0]
    wmin = min(p[3] for row in net for p in row)
    # Cubic entry error <=140u, quadratic <=60u (same recurrence).
    # Tensor product and 12-term accumulation give <4096u*M homogeneous
    # L1 error, padded to 4096u*M. Projection amplifies weight error by
    # 1+max|xyz|/wmin; the full quotient bound is charged explicitly.
    homogeneous_eval = up(4096*2**-53*max_control)
    weight_eval = up(4096*2**-53*max(p[3] for row in net for p in row))
    magnitude = max(sum(abs(v) for v in p[:3])/p[3] for row in net for p in row)
    def project(e,ew):
        if wmin <= ew:
            raise ValueError('unknown projection denominator')
        return up((e+magnitude*ew)/(wmin-ew)+2**-52*magnitude)
    return {'radius':r,'u_knots':[knots[0]]*4+[v for v in knots[1:-1] for _ in range(3)]+[knots[-1]]*4,
            'v_knots':[0.]*3+[1.1]*3,'control_net':net,'weight_lower':wmin,
            'homogeneous_control_enclosure':control_error,'projected_control_enclosure':project(control_error,weight_error),
            'weight_evaluation_bound':weight_eval,'weight_control_enclosure':weight_error,'homogeneous_evaluation_bound':homogeneous_eval,'projected_evaluation_bound':project(homogeneous_eval,weight_eval),
            'fit_truncation':fit,'evaluation_allocation':1e-10,
            'evaluation_budget_pass':project(homogeneous_eval,weight_eval)<1e-10,'full_admission':False}


def main():
    p=argparse.ArgumentParser()
    p.add_argument('--coefficients',type=Path,required=True)
    p.add_argument('--physical',type=Path,required=True)
    p.add_argument('--output',type=Path,required=True)
    a=p.parse_args()
    out=construct(coefficients(a.coefficients),json.loads(a.physical.read_text()))
    a.output.write_text(json.dumps(out,indent=2)+'\n')
    print(json.dumps({k:v for k,v in out.items() if k not in ['control_net','u_knots','v_knots']},indent=2))


if __name__=='__main__':
    main()
