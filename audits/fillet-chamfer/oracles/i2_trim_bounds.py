"""I2 actual-coefficient trim derivative enclosures; no production admission.

All elementary interval operations round out one binary64 ulp. Trigonometric
endpoint values use Taylor polynomials with an explicit remainder, not libm.
Jets contain derivative/factorial. Implicit jets solve H_j=H_t*T_j+rest.
This bounds ideal traces on the represented cones, not Truck's stored NURBS.
"""
import argparse
from functools import lru_cache
import json
import math
from pathlib import Path


def down(x):
    return math.nextafter(x, -math.inf)


def up(x):
    return math.nextafter(x, math.inf)


class I:
    def __init__(self, a, b=None):
        self.a, self.b = float(a), float(a if b is None else b)
        if not math.isfinite(self.a + self.b) or self.a > self.b:
            raise ValueError('nonfinite or reversed enclosure')

    def __add__(self, other):
        o = iv(other)
        return I(down(self.a + o.a), up(self.b + o.b))
    __radd__ = __add__

    def __neg__(self):
        return I(-self.b, -self.a)

    def __sub__(self, other):
        return self + -iv(other)

    def __rsub__(self, other):
        return iv(other) + -self

    def __mul__(self, other):
        o = iv(other)
        v = [self.a*o.a, self.a*o.b, self.b*o.a, self.b*o.b]
        return I(down(min(v)), up(max(v)))
    __rmul__ = __mul__

    def __truediv__(self, other):
        o = iv(other)
        if o.a <= 0 <= o.b:
            raise ValueError('unknown inverse denominator')
        return self * I(down(1/o.b), up(1/o.a))

    def sqrt(self):
        if self.a <= 0:
            raise ValueError('unknown positive radicand')
        return I(down(math.sqrt(self.a)), up(math.sqrt(self.b)))

    def norm(self):
        return max(abs(self.a), abs(self.b))


def iv(x):
    return x if isinstance(x, I) else I(x)


def trig_point(x, cosine):
    # |x| <= 4. Degree 60/61, remainder <= 4^62/62! < 1e-48.
    if abs(x) > 4:
        raise ValueError('trigonometric domain')
    term = I(1) if cosine else I(x)
    result = term
    for j in range(1, 31):
        k = 2*j if cosine else 2*j+1
        term = term * (-I(x)*I(x)) / (k*(k-1))
        result = result + term
    return result + I(-1e-48, 1e-48)


def trig(x, cosine=False):
    if x.a < 0 or x.b > 3.142:
        raise ValueError('trigonometric trim domain')
    a, b = trig_point(x.a, cosine), trig_point(x.b, cosine)
    lo, hi = min(a.a, b.a), max(a.b, b.b)
    # Conservative intervals enclosing pi/2 and pi, including endpoint extrema.
    if not cosine and x.a <= 1.570796326794897 and x.b >= 1.570796326794896:
        hi = 1.
    if cosine and x.a <= 3.141592653589794 and x.b >= 3.141592653589792:
        lo = -1.
    return I(lo, hi)


def const(x):
    return [iv(x)] + [I(0) for _ in range(4)]


def add(a, b):
    return [x+y for x, y in zip(a, b)]


def scale(a, x):
    return [y*x for y in a]


def mul(a, b):
    return [sum((a[j]*b[k-j] for j in range(k+1)), I(0)) for k in range(5)]


def sqrt(a):
    b = [a[0].sqrt()]
    for k in range(1, 5):
        b.append((a[k]-sum((b[j]*b[k-j] for j in range(1, k)), I(0)))/(2*b[0]))
    return b


def inv(a):
    b = [I(1)/a[0]]
    for k in range(1, 5):
        b.append(-sum((a[j]*b[k-j] for j in range(1, k+1)), I(0))/a[0])
    return b


@lru_cache(maxsize=8192)
def sincos(phi):
    s, c = trig(phi), trig(phi, True)
    return [s, c, -s/2, -c/6, s/24], [c, -s, -c/2, s/6, c/24]


def coefficients(path):
    # Decimal strings in retained report enclose exact coefficients recovered from
    # binary64 line endpoints. One ulp encloses decimal-to-binary conversion.
    d = json.loads(path.read_text())['cones']
    def read(name, key, divisor=1):
        x = float(d[name][key])
        return I(down(x), up(x))/divisor
    return read('inner', 'base_radius', 10), read('outer', 'base_radius', 10), read('inner', 'slope'), read('outer', 'slope')


def canal(t, phi, r, coeff):
    _, b, _, m = coeff
    s, c = sincos(phi)
    q = add(const(b-r*(1+m*m).sqrt()), scale(t, -m))
    h = sqrt(add(mul(q, q), const(-r*r)))
    d = sqrt(add(scale(mul(q, q), 1+m*m), const(-r*r)))
    alpha, beta = mul(h, inv(d)), scale(mul(q, inv(d)), m)
    return [add(h, scale(mul(alpha, s), r)), scale(add(const(1), scale(c, -1)), r), add(t, scale(mul(beta, s), r))]


def equation(t, phi, r, coeff, kind):
    p = canal(t, phi, r, coeff)
    if kind == 'BC':
        return add(add(p[2], scale(sincos(phi)[0], r)), const(-r))
    if kind == 'DC':
        return p[2]
    a, _, n, _ = coeff
    return add(add(sqrt(add(mul(p[0], p[0]), mul(p[1], p[1]))), scale(p[2], -n)), const(-a))


def root_box(phi, r, coeff, kind):
    lo, hi = (5-2*r, 5) if kind == 'CI' else (-r-1e-10, r+1e-10)
    sign = -1 if kind == 'CI' else 1
    a0,b0,n,m = coeff
    sine,cosine = sincos(phi)
    s,c = sine[0],cosine[0]
    def value(t):
        q = b0-m*t-r*(1+m*m).sqrt()
        h = (q*q-r*r).sqrt()
        d = ((1+m*m)*q*q-r*r).sqrt()
        x,y,z = h+r*h*s/d, r*(1-c), t+r*m*q*s/d
        result = (x*x+y*y).sqrt()-a0-n*z if kind == 'CI' else z+(r*s-r if kind == 'BC' else 0)
        return result*sign
    if value(lo).b > 0 or value(hi).a < 0:
        raise ValueError('root bracket does not enclose the whole parameter box')
    for lower in [True, False]:
        a, b = lo, hi
        for _ in range(50):
            mid = (a+b)/2
            v = value(mid)
            if (v.b < 0 if lower else v.a <= 0):
                a = mid
            else:
                b = mid
        if lower:
            left = a
        else:
            right = b
    return I(left, right)


def implicit(phi, r, coeff, kind):
    box = root_box(phi, r, coeff, kind)
    # Exact H_t: D0 for BC/DC; -D0*(n+k*Px/rho) for CI.
    p = canal(const(box), phi, r, coeff)
    _, b, n, m = coeff
    q = b-m*box-r*(1+m*m).sqrt()
    h = (q*q-r*r).sqrt()
    k = m*q/h
    L = (1+k*k).sqrt()
    D0 = 1+r*trig(phi)*m*m*r*r/(h*h*h*L*L*L)
    denom = -D0*(n+k*p[0][0]/(p[0][0]*p[0][0]+p[1][0]*p[1][0]).sqrt()) if kind == 'CI' else D0
    if denom.a <= 0 <= denom.b:
        raise ValueError('implicit derivative singularity')
    t = const(box)
    for j in range(1, 5):
        t[j] = -equation(t, phi, r, coeff, kind)[j]/denom
    return canal(t, phi, r, coeff), t, denom


def inner(phi, r, coeff, kind):
    a, _, n, _ = coeff
    R = a+n*r+r*(1+n*n).sqrt()
    s, c = sincos(phi)
    if kind == 'AS':
        return [add(const(R), scale(c, r)), const(0), add(const(r), scale(s, r))]
    y = scale(add(const(1), scale(c, -1)), r)
    rho = add(const(R), scale(c, -r))
    return [sqrt(add(mul(rho, rho), scale(mul(y, y), -1))), y, scale(add(const(1), scale(s, -1)), r)]


def cone_chart_fourth(p):
    # theta'=cross(Pxy,Pxy')/rho^2; integrate jet coefficients.
    dx = [p[0][j+1]*(j+1) for j in range(4)] + [I(0)]
    dy = [p[1][j+1]*(j+1) for j in range(4)] + [I(0)]
    numerator = add(mul(p[0], dy), scale(mul(p[1], dx), -1))
    theta_prime = mul(numerator, inv(add(mul(p[0],p[0]), mul(p[1],p[1]))))
    # eta derivative uses the actual normalized generatrix dz=5.1000002.
    # Rounded lower bound is deliberately smaller than either stored line span.
    return up(6*theta_prime[3].norm()), up(24*p[2][4].norm()/5.10000019)


def stationary_expression(phi, t, r, coeff):
    a,b,n,m = coeff
    q = b-m*t-r*(1+m*m).sqrt()
    k = m/(1-(I(r)/q)*(I(r)/q)).sqrt()
    L = (1+k*k).sqrt()
    sine,cosine = sincos(phi)
    si,co = sine[0],cosine[0]
    x = (q*q-r*r).sqrt()+r*si/L
    y = r*(1-co)
    rho = (x*x+y*y).sqrt()
    radial_x = (1-(y/rho)*(y/rho)).sqrt()
    H = r*((radial_x-n*k)*co/L+y*si/rho)
    lower = (r*si*(n*k-1)/L).a if co.a >= 0 else -math.inf
    return H,lower


def run(coeff, r):
    rows = {}
    # Deliberately extended domains enclose actual endpoints; DC is a small
    # subset, AS lies between pi-atan(n) and pi. No endpoint snapping.
    domains = {'CI': (0, 1.63), 'BC': (0, math.pi/2+1e-14), 'DC': (math.pi/2-1e-14, 1.63), 'AB': (0, math.pi/2+1e-14), 'AS': (2.35, math.pi+1e-14)}
    for kind, (a, b) in domains.items():
        maxima = [0.]*4
        denominator = math.inf
        uv4 = [0., 0.]
        canal_uv4 = [0., 0.]
        stationary_second_lower = math.inf
        boxes = math.ceil((b-a)/.04)
        for i in range(boxes):
            phi = I(a+(b-a)*i/boxes, a+(b-a)*(i+1)/boxes)
            if kind in ['CI', 'BC', 'DC']:
                p, t, d = implicit(phi, r, coeff, kind)
                if kind == 'CI':
                    H,lower = stationary_expression(phi,t[0],r,coeff)
                    if H.a <= 0 <= H.b:
                        stationary_second_lower = min(stationary_second_lower,lower)
                sine,cosine = sincos(phi)
                angular = mul(sine,inv(add(const(1),cosine)))
                canal_uv4 = [max(canal_uv4[0],24*t[4].norm()),max(canal_uv4[1],24*angular[4].norm())]
                denominator = min(denominator, min(abs(d.a), abs(d.b)))
            else:
                p = inner(phi, r, coeff, kind)
            if kind == 'CI':
                uv4 = [max(a,b) for a,b in zip(uv4,cone_chart_fourth(p))]
            for j in range(1, 5):
                maxima[j-1] = max(maxima[j-1], up(sum(x[j].norm() for x in p)*math.factorial(j)))
        # Conservative Hermite derivative remainder <= M4*h^3; position M4*h^4/384.
        spans = math.ceil((b-a)/min(.01, (2e-9*384/maxima[3])**.25, (1e-8/maxima[3])**(1/3)))
        h = up((b-a)/spans)
        rows[kind] = {'domain': [a,b], 'derivative_L1_bounds': maxima, 'implicit_denominator_abs_lower': denominator if math.isfinite(denominator) else None, 'spans': spans, 'position_remainder': up(maxima[3]*h**4/384), 'derivative_remainder': up(maxima[3]*h**3), 'fits_span_cap': spans <= 16384}
        if kind in ['CI','BC','DC']:
            rows[kind]['canal_uv_fourth_bounds_t_tan_half_phi'] = canal_uv4
            rows[kind]['spatial_plus_ideal_canal_uv_truncation'] = up((maxima[3]+1.6*canal_uv4[0]+2*r*canal_uv4[1])*h**4/384)
        if kind == 'CI':
            rows[kind]['stationary_G_phiphi_lower'] = stationary_second_lower
            if stationary_second_lower <= 0:
                raise ValueError('stationary uniqueness not proved')
            rows[kind]['inner_cone_chart_fourth_bounds'] = uv4
            rows[kind]['spatial_plus_cone_chart_truncation'] = up((maxima[3]+30*uv4[0]+7.213*uv4[1])*h**4/384)
    return rows


def separations(coeff, r):
    a,b,n,m = coeff
    left,right = 0.,1.63
    assert stationary_expression(I(left),root_box(I(left),r,coeff,'CI'),r,coeff)[0].b < 0
    assert stationary_expression(I(right),root_box(I(right),r,coeff,'CI'),r,coeff)[0].a > 0
    for _ in range(30):
        mid = (left+right)/2
        d = stationary_expression(I(mid),root_box(I(mid),r,coeff,'CI'),r,coeff)[0]
        if d.b < 0:
            left = mid
        elif d.a > 0:
            right = mid
        else:
            raise ValueError('stationary derivative sign unresolved')
    minimum = root_box(I(left,right),r,coeff,'CI')
    start = root_box(I(0),r,coeff,'CI')
    base = -r*m/(1+m*m).sqrt()
    outer = (b-a)/(m+n)+base
    seam = root_box(I(1.5707963267948965,1.5707963267948968),r,coeff,'BC')
    gaps = [base-seam,start-minimum,outer-start]
    charge = 2*6.4e-9
    if any(g.a <= 4*charge for g in gaps):
        raise ValueError('topological separation budget exhausted')
    return {'stationary_phi':[left,right], 'minimum_t':[minimum.a,minimum.b],
            'gap_intervals':[[g.a,g.b] for g in gaps],
            'gap_order':['base_minus_seam','start_minus_minimum','outer_minus_start'],
            'separation_charge':charge,'comparison_tolerance_used':False,
            'over_budget_control_rejected':any(g.a <= 4*(2*min(x.a for x in gaps)) for g in gaps)}


def stored_controls(coeff, r, rows):
    out = {}
    for kind,row in rows.items():
        a,b = row['domain']
        count = row['spans']
        knots = [a+(b-a)*i/count for i in range(count+1)]
        if count > 16384 or min(y-x for x,y in zip(knots,knots[1:])) <= 0:
            raise ValueError('stored knot span cap or order')
        def endpoint(u):
            if kind in ['CI','BC','DC']:
                return implicit(I(u),r,coeff,kind)[0]
            return inner(I(u),r,coeff,kind)
        previous = endpoint(knots[0])
        max_error,max_control,endpoint_error = 0.,0.,0.
        controls = []
        for u,v in zip(knots,knots[1:]):
            current = endpoint(v)
            h = I(v)-I(u)
            box = [[p[0] for p in previous],
                   [p[0]+p[1]*h/3 for p in previous],
                   [p[0]-p[1]*h/3 for p in current],
                   [p[0] for p in current]]
            span = []
            for j,point in enumerate(box):
                stored = [(x.a+x.b)/2 for x in point]
                error = up(sum(up(max(abs(v-x.a),abs(v-x.b))) for v,x in zip(stored,point)))
                max_error = max(max_error,error)
                if j in [0,3]:
                    endpoint_error = max(endpoint_error,error)
                max_control = max(max_control,sum(abs(v) for v in stored))
                span.append(stored)
            if controls:
                assert span[0] == controls[-1][-1]
            controls.append(span)
            previous = current
        # This is a conservative arithmetic allocation candidate, NOT yet a
        # verified bound for Truck's basis-window evaluator or exported PCurves.
        out[kind] = {'control_enclosure_L1':max_error,'endpoint_enclosure_L1':endpoint_error,
                     'max_control_L1':max_control,'positive_spatial_weights':True,
                     'span_count':count,'controls':controls,'knots':knots,
                     'control_budget_pass':max_error < 1e-10,
                     'endpoint_budget_pass':endpoint_error < 1e-10}
    return out


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--coefficients', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--stored-radius', type=float, choices=[.5,1.,1.5])
    args = parser.parse_args()
    coeff = coefficients(args.coefficients)
    out = {'coefficient_boxes': [[x.a,x.b] for x in coeff], 'radii': {str(r): run(coeff,r) for r in [.5,1.,1.5]}, 'full_admission': False, 'unmet': 'Truck stored control, chart composition, shared endpoint and same-parameter evaluation bounds are not established by derivative enclosures.'}
    out['separations'] = {str(r):separations(coeff,r) for r in [.5,1.,1.5]}
    if args.stored_radius is not None:
        out['stored'] = stored_controls(coeff,args.stored_radius,out['radii'][str(args.stored_radius)])
    rejected = []
    for a,b in [(-1,1),(0,0)]:
        try:
            I(1)/I(a,b)
        except ValueError:
            rejected.append(True)
    assert len(rejected) == 2
    assert all(row['fits_span_cap'] and row['position_remainder'] <= 2e-9 and row['derivative_remainder'] <= 1e-8 for rows in out['radii'].values() for row in rows.values())
    out['zero_denominator_controls_rejected'] = rejected
    out['full_admission'] = False
    out['unmet'] = ('The stored cubics cover extended mathematical domains. Physical cross-face shared endpoints, '
                    'stored UV controls on every support, Truck basis-window and support evaluation error, '
                    'and composed same-parameter discrepancies are not yet established. No builder admission.')
    args.output.write_text(json.dumps(out, indent=2)+'\n')
    print(json.dumps({k:v for k,v in out.items() if k != 'stored'}, indent=2))


if __name__ == '__main__':
    main()
