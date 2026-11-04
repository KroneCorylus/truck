"""Independent retained-material and full divergence-flux oracle for bounded F9.

The canonical orthonormal coordinates are u=(x+5z)/sqrt(26), y=y,
d=(50-z+5x)/sqrt(26). Retain a(d)<=y<=b(d) and l-5d<=u<=h(d).
The two lower-depth arcs are cylinder constraints; their crease stops at dC,
and the ruling-cylinder/cap ellipse continues to dT>dC. Native mass properties
are deliberately not inputs. Quadrature convergence is numerical evidence,
not an interval-certified bound.
"""
import math,json
import numpy as np
from numpy.polynomial.legendre import leggauss
K=math.sqrt(26); D=50/K; L=770/K; l=250/K

def geometry(r):
 lo,hi=-100.,0.
 def f(t):return (50-t*t/200)/K
 def fp(t):return -t/(100*K)
 def g(t):return f(t)-r/math.sqrt(1+fp(t)**2)-r
 assert g(lo)<0<g(hi)
 for _ in range(64):
  t=(lo+hi)/2
  if g(t)>0:hi=t
  else:lo=t
 t=(lo+hi)/2;p=fp(t);dT=f(t);Y=t+r*p/math.sqrt(1+p*p);U=L-(5+K)*r;dC=r*(1+5/K)
 return t,dT,Y,U,dC

def calc(r,n):
 t,dT,Y,U,dC=geometry(r);q,w=leggauss(n);theta=(q+1)*math.pi/4;w=w*math.pi/4
 area=np.zeros(3);flux=0;volume=0
 for lo,hi in zip([0,dC,dT],[dC,dT,D]):
  d=lo+(hi-lo)*np.sin(theta)**2;weights=w*2*(hi-lo)*np.sin(theta)*np.cos(theta)
  b=np.sqrt(10000-200*K*d);bp=-100*K/b
  if hi<=dT:
   v=np.sqrt(r*r-(d-r)**2);a=Y-v;ap=(d-r)/v
  else:a=-b;ap=-bp
  if hi<=dC:
   v=np.sqrt(r*r-(d-r)**2);h=U+v;hp=-(d-r)/v
  else:h=L-5*d;hp=np.full_like(d,-5.)
  low=l-5*d;width=b-a;length=h-low
  # Flux on four side graphs; first moments integrate the other coordinate exactly.
  normals=[np.stack([-width,np.zeros_like(d),-5*width],1),np.stack([width,np.zeros_like(d),-hp*width],1),np.stack([np.zeros_like(d),-length,ap*length],1),np.stack([np.zeros_like(d),length,-bp*length],1)]
  positions=[np.stack([low,(a+b)/2,d],1),np.stack([h,(a+b)/2,d],1),np.stack([(h+low)/2,a,d],1),np.stack([(h+low)/2,b,d],1)]
  for N,X in zip(normals,positions):
   area+=weights@N;flux+=float(weights@np.sum(N*X,axis=1))/3
  volume+=float(weights@(width*length))
 width0=100-Y;length0=U-l;area[2]-=width0*length0
 return dict(radius=r,order=n,contact_y=t,dT=dT,dC=dC,Y=Y,U=U,seam_end=[U+r/K,Y-r/K,dC],volume=volume,flux=flux,area_residual=area.tolist(),removed=400000/3-volume)
if __name__=='__main__':
 for r in [.5,1.,1.5]:
  rows=[calc(r,n) for n in [32,64,128]]
  assert max(v['volume'] for v in rows)-min(v['volume'] for v in rows)<2e-9
  for row in rows:
   assert r<row['dC']<row['dT']<D
   assert row['U']>l
   assert abs(row['volume']-row['flux'])<5e-8
   assert max(map(abs,row['area_residual']))<1.5e-8
 for r in [.0001,.01,.5,1,1.5]:
  for n in [16,32,64,128]: print(json.dumps(calc(r,n)))
