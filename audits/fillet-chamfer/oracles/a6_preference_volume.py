"""Independent whole-volume quadrature for the A6 preferred cap-edge family.
Run with uv run --with scipy --with numpy; optional argument is quadrature tolerance.
Reflection of either local corner proves all four cap choices have this volume.
"""

import sys

EPS = float(sys.argv[1]) if len(sys.argv) > 1 else 2e-9
import math, json
from scipy.integrate import quad
import numpy as np

# Qy: t=1-x/d, w=(y-d*t)/h, z=(d*t+h*w*(1-t-w))/(t+w).
# This is the independent bilinear elimination; the remote P/Q/R planes
# and circular/conical cross sections close the entire solid.
a = b = 0.2
h = 0.1
t = math.tan(2 * math.asin(0.1))
cy = t / math.hypot(1, t)
cz = 1 / math.hypot(1, t)
rz = math.sqrt(0.8**2 - b*b)
p, q, r = np.array([a,cy,cz]), np.array([a,0,0.8]), np.array([0,b,rz])
n = np.cross(r-p,q-p)
c = n @ p
s = (cz-0.8)/cy

def area(x):
    rad = 0.8 + min(x, a)
    lo = max(b-x,0)
    planes = [(-s,1,0.8),(1,-s,0.8),(-1,-1,-h)]
    for swap in [False,True]:
        m = n[[0,2,1]] if swap else n
        planes.append((m[1],m[2],c-m[0]*x))
    def height(y):
        low, high = lo, math.sqrt(max(0,rad*rad-y*y))
        if x < a and y < lo+h*x/a:
            t0=1-x/a
            w=(y-lo)/h
            low=max(low,(a*t0+h*w*(1-t0-w))/(t0+w))
        for u,v,w in planes:
            z=(w-u*y)/v
            if v>0: high=min(high,z)
            else: low=max(low,z)
        return max(0,high-low)
    lines=[(0,lo)]+[(-u/v,w/v) for u,v,w in planes]
    breaks=[lo+h*min(x/a,1)]
    for i,(m,c0) in enumerate(lines):
        for slope,offset in lines[:i]:
            if m != slope: breaks.append((offset-c0)/(m-slope))
        discriminant=(m*c0)**2-(1+m*m)*(c0*c0-rad*rad)
        if discriminant>=0:
            for sign in [-1,1]: breaks.append((-m*c0+sign*math.sqrt(discriminant))/(1+m*m))
    points=[]
    for y in sorted(set(y for y in breaks if lo+1e-12<y<rad-1e-12)):
        if not points or y-points[-1]>1e-12: points.append(y)
    return quad(height,lo,rad,points=points,epsabs=EPS/10,epsrel=EPS,limit=500)[0]
volume=2*quad(area,0,a,epsabs=EPS,epsrel=EPS,limit=500)[0]+(1-2*a)*area(a)
print(json.dumps(dict(volume=volume,quadrature_tolerance=EPS)))
