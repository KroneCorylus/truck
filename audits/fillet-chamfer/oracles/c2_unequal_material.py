"""Independent unequal C2 oracle; sums both cap laws at every axial point.
Rectangle volume bounds use nested sections in each nonoverlapping cap layer.
The floating allowance scales with this profile; it is not directed rounding.
"""
import math



def section(p, cap, t):
    assert sum(max(v for u, v in cap) for cap in p['setbacks']) < p['length']
    z = t if cap == 0 else p['length'] - t
    offsets = [sum(u * max(1 - distance / v, 0.)
                   for (u, v), distance in zip(pair, (z, p['length'] - z)))
               for pair in zip(*p['setbacks'])]
    left, high, bottom, right, chain = offsets
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
    cap, t = 0, z
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
    depth = [max(v for u,v in ds) for ds in p['setbacks']]
    assert sum(depth) < p['length']
    lower = upper = area(p, 0, depth[0])*(p['length']-sum(depth))
    for cap in range(2):
        breaks = sorted(set([0.] + [v for u,v in p['setbacks'][cap]]))
        for a, b in zip(breaks, breaks[1:]):
            dx = (b-a)/subdivisions
            values = [area(p, cap, a+(b-a)*i/subdivisions) for i in range(subdivisions+1)]
            assert all(y >= x-1e-10 for x, y in zip(values, values[1:]))
            lower += dx*math.fsum(values[:-1])
            upper += dx*math.fsum(values[1:])
    # Each primitive's terms are <= pi*r^2; subtractive cancellation affects
    # absolute error. Charge 4096 ulps of that scale per unit axial length.
    extent = max(p['outer_radius'], p['inner_radius'], *map(abs, p['center']),
                 abs(p['left']), abs(p['right']), abs(p['bottom']), abs(p['high']))
    rounding = 4096 * math.ulp(math.pi * extent**2) * p['length']
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
        for t in sorted({0., *[v for u,v in p['setbacks'][cap]], *[v*f for u,v in p['setbacks'][cap] for f in (.25,.5,.75)]}):
            samples.append({'cap': cap, 't': t, 'points': contacts(p, cap, t)})
    return {'parameters': p, 'volume_lower': lo, 'volume_upper': hi,
            'volume_midpoint': (lo+hi)/2, 'floating_allowance': rounding,
            'source_volume': area(p, 0, max(v for u,v in p['setbacks'][0]))*p['length'],
            'contacts': samples}


