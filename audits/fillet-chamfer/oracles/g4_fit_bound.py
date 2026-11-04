"""Rational continuous remote-trim derivative bound, independent of Truck geometry.

The fit uses theta, not depth, so the square-root endpoint is regular.
This certifies the analytic Hermite truncation term; floating-point construction
uses the separate local arithmetic allowance in g4.rs.
"""
from fractions import Fraction as Q
from math import comb
import json


def derivative(p):
    return [i * v for i, v in enumerate(p)][1:]


def bernstein(p, a=Q(949, 1000), b=Q(1001, 1000)):
    n = len(p) - 1
    c = [sum(p[j] * comb(j, k) * a ** (j-k) * (b-a) ** k
             for j in range(k, n+1)) for k in range(n+1)]
    return [sum(c[k] * Q(comb(i, k), comb(n, k)) for k in range(i+1))
            for i in range(n+1)]


def certificate():
    f = list(map(Q, [0, 800, -1350, 1100, -550]))
    x = list(map(Q, [0, 1040, -810, 800, -400]))
    assert Q(1122, 100)**2 < 126 < Q(1123, 100)**2
    assert Q(2861, 100)**2 < 819 < Q(2862, 100)**2
    values = []
    for order in range(1, 5):
        f, x = derivative(f), derivative(x)
        fb, xb = bernstein(f), bernstein(x)
        if order == 1:
            assert max(fb) / Q(1123, 100) < -50
        assert max(map(abs, fb)) / Q(1122, 100) < 2000
        assert max(map(abs, xb)) / Q(2861, 100) < 4000
        values.append(dict(order=order, f_upper=float(max(map(abs, fb))/Q(1122,100)),
                           x_upper=float(max(map(abs, xb))/Q(2861,100))))
    radius = Q(3, 2)
    t1 = radius / 50
    t2 = (radius + 2000*t1**2)/50
    t3 = (radius + 6000*t1*t2 + 2000*t1**3)/50
    t4 = (radius + 2000*(4*t1*t3+3*t2**2+6*t1**2*t2+t1**4))/50
    x4 = 4000*(t4+4*t1*t3+3*t2**2+6*t1**2*t2+t1**4)
    assert 1 + Q(26,819) < Q(102,100)**2
    bound = Q(102,100)*x4 + Q(102,100)*radius + radius
    assert bound < 10000
    return dict(polynomial_bounds=values, inverse_derivatives=list(map(float,[t1,t2,t3,t4])),
                remote_fourth_derivative_upper=float(bound), production_bound=10000,
                hermite_error='10000 * scale * theta_span**4 / 384')

if __name__ == '__main__':
    print(json.dumps(certificate(), indent=2))

# Represented-support budget used by g4.rs:
# B(t)=sum_i Bernstein_i(t) P_i implies sup|B-Q| <= max_i|P_i-Q_i|.
# For S(t,h)=B(t)+hV and axial s=V.u, h in [h0,h1], its discrepancy
# from the projected u-sweep is bounded by the base axial residual plus
# max(|h0|,|h1|)*|V-su|*(1+|alpha|+|beta|). No term is discarded as tiny.
# Plane residuals are affine: the four corners of each retained-domain rectangle
# bound the full domain. These and cap control deviations form representation.
# On the certified remote interval, m=min|F'|>0 gives |dt|<=|F(t)-d|/m.
# Interval polynomial evaluation propagates this to endpoint value and derivative.
# Taylor sin/cos polynomials through degree 25 have remainder <= |theta|^26/26!.
# For each STORED span h, ideal Bezier controls are P0, P0+hV0/3,
# P1-hV1/3, P1. Comparing their interval enclosures with stored controls bounds
# endpoint/inverse/control-construction error by convexity. Production sums
# M4*h^4/384 + control_error + evaluation_error + contact_error + representation + arithmetic
# and requires this <= eps/2. eps and the 256*machine_epsilon arithmetic factor
# are unchanged. The arithmetic allowance reserves frame and curve evaluation
# errors. evaluation_error=gamma(32)*max(stored_control_norm) bounds three cubic
# de Boor interpolation levels, including weights. The frame arithmetic term is
# separate; this script proves the analytic term, not the whole implementation.
# Frame/plane terms: with delta=3*max|A^T A-I| (including dot rounding),
# polar decomposition bounds distance to an orthonormal frame by delta for
# delta<1. Multiply by the admitted local-coordinate L1 extent. Plane normal
# cross-product error E=16*u*|a|*|b| is charged through 2E/(|a cross b|-E),
# plus normalization rounding, multiplied by the whole-domain distance to the
# plane origin. Near-degenerate normals and nonfinite budgets reject.
