import numpy as np
from a5_procedural_volume import modes

def inside(q,p,hole=False,both=False):
 x,y,z=q;hx,hy,by,ay,bx,ax=p
 if not 0<z<1:return False
 t=min(z,1-z) if both else 1-z;sx=bx*max(0,1-t/ax);sy=by*max(0,1-t/ay);s0=.03*max(0,1-t/.03)
 if not sy<y<1.1-s0 or x>=1.1-s0:return False
 if x>1 and y>1 and (x-1)**2+(y-1)**2>=(.1-s0)**2:return False
 if hole and .3<x<.5 and .3<y<.5:return False
 if y>=hy:return x>sx
 if t>=max(ax,ay):return x>hx*(1-y/hy)
 lo=max(0,(t-ax)/(ay-ax)) if ay>ax else 0
 hi=min(1,(ax-t)/(ax-ay)) if ax>ay else 1
 def P(w):
  u=1-t/(ax+(ay-ax)*w)
  return (u*(1-w)*bx+w*hx,(1-w)*hy+u*w*by)
 start,end=P(lo),P(hi)
 if y>start[1] or y<end[1]:return x>hx*(1-y/hy)
 for _ in range(60):
  mid=(lo+hi)/2
  if P(mid)[1]>y:lo=mid
  else:hi=mid
 return x>P((lo+hi)/2)[0]

if __name__ == "__main__":
 import itertools
 count=0
 for p in modes.values():
  hx,hy,by,ay,bx,ax=p
  A,B,C,D=map(np.array,[[0,hy,ax],[bx,hy,0],[hx,by,0],[hx,0,ay]])
  for u,w in itertools.product([.2,.5,.8],repeat=2):
   P=(1-u)*(1-w)*A+u*(1-w)*B+u*w*C+(1-u)*w*D
   n=-np.cross((1-w)*(B-A)+w*(C-D),(1-u)*(D-A)+u*(C-B))
   n/=np.linalg.norm(n)
   for e in [-1e-4,1e-4]:
    q=P+e*n;q[2]=1-q[2]
    assert inside(q,p)==(e>0)
    count+=1
  assert inside([.4,.4,.5],p) and not inside([.4,.4,.5],p,hole=True)
  assert inside([.2,.2,.5],p,hole=True,both=True)
 print(f"{count} independent patch-side witnesses passed; hole/interior controls passed")
