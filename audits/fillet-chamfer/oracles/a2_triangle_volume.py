"""Independent quarter-cylinder cross-section integral for the bounded triangle family.

At axial depth x, intersect the exact cone disk with the original radial planes,
straight bevel lines, lip lines and independent P/Q/R corner planes. Apply the
T/U/V material plane only in 0 <= x <= a. The middle section is constant.
No production corner helpers or native output are used. This is numerical
quadrature, not a formal enclosure. Run with uv run --with scipy --with numpy.
Optional argv[1] sets absolute quadrature tolerance for convergence checks.
"""

import sys

EPS = float(sys.argv[1]) if len(sys.argv) > 1 else 2e-9
import math, json
from scipy.integrate import quad
import numpy as np

out = []
angle = math.pi / 6
k = math.tan(angle)
z_angle = (0.8 * k * k + math.sqrt(1 + k * k * (1 - 0.8**2))) / (1 + k * k)
t_angle = k * (z_angle - 0.8) / z_angle
for a, t in [(0.1, math.tan(2 * math.asin(0.05))), (0.2 * k, t_angle)]:
    b = 0.2
    cy = t / math.hypot(1, t)
    cz = 1 / math.hypot(1, t)
    rz = math.sqrt(0.8**2 - b * b)
    p = np.array([a, cy, cz])
    q = np.array([a, 0, 0.8])
    r = np.array([0, b, rz])
    n = np.cross(r - p, q - p)
    c = n @ p
    s = (cz - 0.8) / cy

    def area(x):
        rad = 0.8 + b * min(x / a, 1)
        lo = b * max(1 - x / a, 0)
        planes = [(-s, 1, 0.8), (1, -s, 0.8), (-b / a, -1, -b)]
        for swap in [False, True]:
            m = n[[0, 2, 1]] if swap else n
            planes.append((m[1], m[2], c - m[0] * x))
        if x <= a:
            planes.append((-a * b, -a * a, -a * b * (a + b) + b * b * x))

        def height(y):
            low = lo
            high = math.sqrt(max(0, rad * rad - y * y))
            for u, v, w in planes:
                z = (w - u * y) / v
                if v > 0:
                    high = min(high, z)
                else:
                    low = max(low, z)
            return max(0, high - low)

        # Split at all line/line and line/circle crossings, including inactive ones.
        lines = [(0, lo)] + [(-u / v, w / v) for u, v, w in planes]
        breaks = []
        for i, (m, c0) in enumerate(lines):
            for slope, offset in lines[:i]:
                if m != slope:
                    breaks.append((offset - c0) / (m - slope))
            discriminant = (m * c0) ** 2 - (1 + m * m) * (c0 * c0 - rad * rad)
            if discriminant >= 0:
                for sign in [-1, 1]:
                    breaks.append(
                        (-m * c0 + sign * math.sqrt(discriminant)) / (1 + m * m)
                    )
        points = sorted(set(y for y in breaks if lo < y < rad))
        return quad(
            height, lo, rad, points=points, epsabs=EPS / 10, epsrel=EPS, limit=250
        )[0]

    v = 2 * quad(area, 0, a, epsabs=EPS, epsrel=EPS, limit=250)[0] + (1 - 2 * a) * area(
        a
    )
    out.append(dict(a=a, t=t, volume=v, quadrature_tolerance=EPS))
print(json.dumps(out, indent=2))
