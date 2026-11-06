"""Independent I2 ideal material witnesses and regularized section-volume oracle.

Ported from the independently reviewed 2026-11-04 ideal contract. No B-rep,
Truck result, native result, or mesh is read. Quadrature convergence is evidence,
not a rigorous enclosure; its numerical allowance is separate from fit error.
Coordinates use a=20,b=30 and place the revolution axis at X=Y=0.
"""
import argparse
import json
import math
from pathlib import Path
S=math.sqrt(2)
def bisect(f,lo,hi):
 fl,fh=f(lo),f(hi)
 assert fl*fh<=0,(lo,hi,fl,fh)
 for _ in range(48):
  m=(lo+hi)/2;fm=f(m)
  if fl*fm<=0:hi=m;fh=fm
  else:lo=m;fl=fm
 return [lo,hi,fl,fh]
def mid(box):return (box[0]+box[1])/2
def coeff(t,r):
 q=30-t-S*r;h=math.sqrt(q*q-r*r);k=q/h;L=math.sqrt(1+k*k)
 return q,h,k,L
def point(t,phi,r):
 q,h,k,L=coeff(t,r);return [h+r*math.sin(phi)/L,r*(1-math.cos(phi)),t+r*k*math.sin(phi)/L]
def seam(phi,r):return bisect(lambda t:t-r+r*math.sin(phi)*(1+coeff(t,r)[2]/coeff(t,r)[3]),-r,r)
def remote(phi,r):return bisect(lambda t:math.hypot(*point(t,phi,r)[:2])-point(t,phi,r)[2]-20,5-2*r,5)
def phi_outer(t,r):return math.acos(-r/(S*coeff(t,r)[0]))
def gauss(n):
 out=[]
 for i in range(1,n+1):
  x=math.cos(math.pi*(i-.25)/(n+.5))
  for _ in range(20):
   a,b=1.,x
   for j in range(2,n+1):a,b=b,((2*j-1)*x*b-(j-1)*a)/j
   d=n*(x*b-a)/(x*x-1);dx=b/d;x-=dx
   if abs(dx)<1e-15:break
  out.append((x,2/((1-x*x)*d*d)))
 return out
def quad(f,a,b,g):return (b-a)/2*sum(w*f((a+b)/2+(b-a)/2*x) for x,w in g)
def cut(z,phi,r):
 s=math.sin(phi);t=z-r*s/S
 for _ in range(5):
  q,h,k,L=coeff(t,r)
  t-=(t+r*s*k/L-z)/(1+r*s*r*r/(h**3*L**3))
 q,h,k,L=coeff(t,r)
 return h+r*s/L
# Independent section quadrature: solve the inner-cone crossing before integrating,
# and remove the square-root endpoint in z with z=r*(1-cos(u)).
def area(z,r,g):
 U=30-z;A=20+z if z>=r*(1+1/S) else 20+(1+S)*r-math.sqrt(max(0,2*r*z-z*z))
 B=0 if z>=r else r-math.sqrt(max(0,2*r*z-z*z))
 def prim(R,y):return .5*(y*math.sqrt(R*R-y*y)+R*R*math.asin(y/R))
 base=math.pi*(U*U-A*A)/4-prim(U,B)+prim(A,B)
 lo=math.acos(1-B/r);yc=r+r*r/(S*(30-z-r/S));hi=math.acos(1-yc/r)
 def gap(phi):
  y=r*(1-math.cos(phi));return cut(z,phi,r)-math.sqrt(A*A-y*y)
 split=lo
 if gap(lo)<0 and gap(hi)>0:split=mid(bisect(gap,lo,hi))
 elif gap(hi)<=0:split=hi
 def removed(phi,empty):
  y=r*(1-math.cos(phi));x=math.sqrt(A*A-y*y) if empty else cut(z,phi,r)
  return (math.sqrt(U*U-y*y)-x)*r*math.sin(phi)
 return base-quad(lambda v:removed(v,True),lo,split,g)-quad(lambda v:removed(v,False),split,hi,g)
def vol(r,n):
 g=gauss(n);ts=((30-S*r)**2-400-r*r)/(2*(50-S*r))
 breaks=[r,r*(1+1/S),ts,5-r/S,5]
 return quad(lambda u:area(r*(1-math.cos(u)),r,g)*r*math.sin(u),0,math.pi/2,g)+sum(quad(lambda z:area(z,r,g),a,b,g) for a,b in zip(breaks,breaks[1:]))
def inside(P,r):
 x,y,z=P;rho=math.hypot(x,y)
 if min(x,y,z)<0 or z>5 or rho<20+z or rho>30-z:return False
 if z<r*(1+1/S) and rho<20+(1+S)*r-math.sqrt(max(0,r*r-(z-r)**2)):return False
 if z<r and y<r-math.sqrt(max(0,r*r-(z-r)**2)):return False
 yc=r+r*r/(S*(30-z-r/S))
 if y<yc:
  s=math.sqrt(max(0,1-(y/r-1)**2));t=mid(bisect(lambda t:t+r*s*coeff(t,r)[2]/coeff(t,r)[3]-z,z-r,z))
  _,h,k,L=coeff(t,r)
  if x>h+r*s/L:return False
 return True
rows=[]
def check(P,n,shift,r,family):
 for requested_eps in [1e-5,1e-4]:
  eps=min(requested_eps,(30-P[2]-math.hypot(*P[:2]))/100)
  assert eps>1e-12
  for sign in [-1,1]:
   Q=[P[i]+eps*(shift[i]+sign*n[i]) for i in range(3)]
   assert min(Q)>0 and Q[2]<5 and 20+Q[2]<math.hypot(*Q[:2])<30-Q[2], (family,Q)
   actual=inside(Q,r);expected=sign<0
   rows.append(dict(r=r,family=family,P=P,Q=Q,eps=eps,requested_eps=requested_eps,sign=sign,expected=expected,actual=actual))
for r in [.5,1.,1.5]:
 for phi in [.2,.7,1.3]:
  t=mid(seam(phi,r));P=point(t,phi,r);_,h,k,L=coeff(t,r)
  # Outward C plus outward B, both normals have negative Y.
  n=[math.sin(phi)/L,-2*math.cos(phi),k*math.sin(phi)/L-math.sin(phi)]
  check(P,n,[0,0,0],r,'outer_seam')
  z=r*(1-math.sin(phi));y=r*(1-math.cos(phi));rho=20+S*r+y;x=math.sqrt(rho*rho-y*y)
  n=[-math.cos(phi)*x/rho,-math.cos(phi)*(y/rho+1),-2*math.sin(phi)]
  check([x,y,z],n,[0,0,0],r,'inner_seam')
 for v in [.1,.4,.7]:
  P=[20+(1+S)*r-r*math.cos(v),0,r+r*math.sin(v)]
  check(P,[-math.cos(v),0,math.sin(v)],[0,.01,0],r,'inner_start_residual')
 end=phi_outer(-r/S,r)
 for u in [.2,.5,.8]:
  phi=math.pi/2+u*(end-math.pi/2)
  t=mid(bisect(lambda t:point(t,phi,r)[2],-r,0));P=point(t,phi,r);P[2]=0
  check(P,[1,0,0],[0,0,.01],r,'outer_base_residual')
 po=phi_outer(5-r/S,r)
 for phi in [.02,.3,.8,po-.01]:
  t=mid(remote(phi,r));P=point(t,phi,r);rho=math.hypot(*P[:2]);_,h,k,L=coeff(t,r)
  n=[math.sin(phi)/L,-math.cos(phi),k*math.sin(phi)/L]
  check(P,n,[2*P[0]/rho,2*P[1]/rho,-2],r,'remote_inner_cone')

assert all(v['actual'] == v['expected'] for v in rows)
parser = argparse.ArgumentParser()
parser.add_argument('--output', type=Path, required=True)
parser.add_argument('--nodes', type=int, default=96)
args = parser.parse_args()
if args.nodes not in [48,96,192]:
    raise ValueError('use the reviewed convergence orders 48,96,192')
volumes = {str(r): vol(r,args.nodes) for r in [.5,1.,1.5]}
reference = {'0.5':970.934499167348,'1.0':937.7409690789835,'1.5':881.0404265627494}
allowance = 1e-8
if args.nodes >= 96:
    assert all(abs(volumes[r]-reference[r]) < allowance for r in volumes)
result = {'witnesses':rows,'volumes':volumes,'nodes':args.nodes,
          'numerical_volume_allowance':allowance,
          'independent_numerical_face_flux_executed':False}
args.output.write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps({'witness_count':len(rows),'failures':0,'volumes':volumes,'numerical_volume_allowance':allowance},indent=2))
