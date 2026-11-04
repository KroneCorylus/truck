"""Independent closed C2 material, using only input supports and selected setbacks.

The section roof is partitioned by x, not an intersection of cylinder halfspaces.
Volume bounds use monotone lower/upper rectangle sums on each setback interval.
A growing inward cap coordinate relaxes every roof/wall constraint, so section
area is nondecreasing. The two disjoint cap layers are integrated separately.
The floating-point allowance is conservative error accounting, not directed-
rounding interval arithmetic. Native geometry is never an expected-value source.
"""
import argparse
import json
import math
from pathlib import Path

PARAMETERS = {
    'outer_radius': 18.75, 'inner_radius': 0.5,
    'center': [19.0998036818876, 2.39999985694885],
    'length': 10.75, 'left': 16.8180108173766,
    'right': 25.443289939164, 'bottom': 0.794433603322526,
    'high': 4.05306587080493,
    'setbacks': [[0.2, 0.6, 0.3, 0.4, 0.4], [0.2, 0.6, 0.3, 0.2, 0.5]],
}


def section(p, cap, t):
    left, high, bottom, right, chain = [max(d-t, 0.) for d in p['setbacks'][cap]]
    cx, cy = p['center']
    ro, ri = p['outer_radius']-chain, p['inner_radius']+chain
    a, b = p['left']+left, p['right']-right
    low, high, flat = p['bottom']+bottom, p['high']-high, cy-ri
    q, s = math.sqrt(ro*ro-high*high), ro*cx/math.hypot(cx, cy)
    assert a < q < s < cx < b and low < flat < high and ro > 0 and ri > 0
    return a, b, low, high, flat, ro, ri, q, s


def roof(p, x, cap, t):
    a, b, low, high, flat, ro, ri, q, s = section(p, cap, t)
    if x <= q:
        return high
    if x <= s:
        return math.sqrt(ro*ro-x*x)
    cx, cy = p['center']
    if x < cx:
        return cy-math.sqrt(ri*ri-(x-cx)**2)
    return flat


def inside(p, point):
    x, y, z = point
    if not 0 <= z <= p['length']:
        return False
    cap = int(z > p['length']/2)
    t = p['length']-z if cap else z
    a, b, low, *_ = section(p, cap, t)
    return a <= x <= b and low <= y <= roof(p, x, cap, t)


def primitive(x, radius):
    return (x*math.sqrt(max(0., radius*radius-x*x)) + radius*radius*math.asin(x/radius))/2


def area(p, cap, t):
    a, b, low, high, flat, ro, ri, q, s = section(p, cap, t)
    cx, cy = p['center']
    return (high*(q-a) + primitive(s, ro)-primitive(q, ro)
            + cy*(cx-s)+primitive(s-cx, ri) + flat*(b-cx)-low*(b-a))


def volume_bounds(p, subdivisions=65536):
    depth = [max(ds) for ds in p['setbacks']]
    assert sum(depth) < p['length']
    lower = upper = area(p, 0, depth[0])*(p['length']-sum(depth))
    for cap in range(2):
        breaks = sorted(set([0.] + p['setbacks'][cap]))
        for a, b in zip(breaks, breaks[1:]):
            dx = (b-a)/subdivisions
            values = [area(p, cap, a+(b-a)*i/subdivisions) for i in range(subdivisions+1)]
            assert all(y >= x-1e-10 for x, y in zip(values, values[1:]))
            lower += dx*math.fsum(values[:-1])
            upper += dx*math.fsum(values[1:])
    # Each primitive's terms are <= pi*r^2; subtractive cancellation affects
    # absolute error. Charge 4096 ulps of that scale per unit axial length.
    rounding = 4096*math.ulp(math.pi*max(p['outer_radius'], p['inner_radius'])**2)*p['length']
    return lower-rounding, upper+rounding, rounding


def contacts(p, cap, t):
    a, b, low, high, flat, ro, ri, q, s = section(p, cap, t)
    cx, cy = p['center']
    z = p['length']-t if cap else t
    return [[x, y, z] for x, y in [(a, low), (a, high), (q, high),
            (s, ro*cy/math.hypot(cx, cy)), (cx, flat), (b, flat), (b, low)]]


def report(p):
    lo, hi, rounding = volume_bounds(p)
    samples = []
    for cap in range(2):
        for t in [0., .1, .25, .35, .45, .55, .6]:
            samples.append({'cap': cap, 't': t, 'points': contacts(p, cap, t)})
    return {'parameters': p, 'volume_lower': lo, 'volume_upper': hi,
            'volume_midpoint': (lo+hi)/2, 'floating_allowance': rounding,
            'source_volume': area(p, 0, max(p['setbacks'][0]))*p['length'],
            'contacts': samples}


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--output', type=Path)
    args = parser.parse_args()
    result = json.dumps(report(PARAMETERS), indent=2)+'\n'
    if args.output:
        args.output.write_text(result)
    else:
        print(result, end='')
