import numpy as np,json,math
from numpy.polynomial.legendre import leggauss
from pathlib import Path

# Procedural rounded rectangle from prior procedural.rs, translated to [0,1.1]^2.
# Independent cross-section boundary integral; no native geometry or volume input.
def integral(f,a,b,n):
 if b<=a:return 0.
 x,w=leggauss(n);return sum(wi*f((a+b)/2+(b-a)*xi/2) for xi,wi in zip(x,w))*(b-a)/2
def correction(t,hx,hy,by,ay,bx,ax,n):
 sx=bx*max(0,1-t/ax);sy=by*max(0,1-t/ay)
 if t>=max(ax,ay):return hx*hy/2
 lo,hi=0.,1.
 if ax!=ay:
  if ay>ax:lo=max(0.,(t-ax)/(ay-ax))
  else:hi=min(1.,(ax-t)/(ax-ay))
 def point(v):
  a=ax+(ay-ax)*v;u=1-t/a
  return np.array([u*(1-v)*bx+v*hx,(1-v)*hy+u*v*by])
 def xdy(v):
  a=ax+(ay-ax)*v;u=1-t/a;du=t*(ay-ax)/a**2
  return point(v)[0]*(-hy+u*by+v*by*du)
 p,q=point(lo),point(hi)
 # Curve plus surviving pivot portions, closed through offset lower-left corner.
 points=[np.array([sx,hy]),p];s=0.
 for a,b in zip(points,points[1:]):s+=(a[0]+b[0])*(b[1]-a[1])/2
 s+=integral(xdy,lo,hi,n)
 points=[q,np.array([hx,sy]),np.array([sx,sy]),np.array([sx,hy])]
 for a,b in zip(points,points[1:]):s+=(a[0]+b[0])*(b[1]-a[1])/2
 return -s
modes={'equal':(.05,.05,.03,.03,.02,.02),'DD':(.04,.05,.03,.03,.04,.02),'DA':(.05/math.sqrt(3),.05,.03,.03,.02*math.sqrt(3),.02)}
def volume(params,n,hole=False,both=False):
 hx,hy,by,ay,bx,ax=params;H=1.;L=1.1;r=.1;a0=.03;b0=.03
 def area(z):
  t=min(z,H-z) if both else H-z
  sx=bx*max(0,1-t/ax);sy=by*max(0,1-t/ay);s0=b0*max(0,1-t/a0)
  base=(L-sx-s0)*(L-sy-s0)-(1-math.pi/4)*(r-s0)**2
  return base-correction(t,*params,n)-(.04 if hole else 0)
 breaks=sorted(set([0.,H]+[H-a for a in [ax,ay,a0]]+([ax,ay,a0] if both else [])))
 return sum(integral(area,a,b,n) for a,b in zip(breaks,breaks[1:]))

if __name__ == "__main__":
 results = {}
 for mode,p in modes.items():
  for hole in (False,True):
   for both in (False,True):
    a,b=volume(p,12,hole,both),volume(p,24,hole,both)
    assert abs(a-b)<1e-12
    results[f"{mode}/{hole}/{both}"]={"volume":b,"quadrature_delta":b-a}
 print(json.dumps(results,indent=2))
