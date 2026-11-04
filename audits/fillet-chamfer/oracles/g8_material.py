"""Independent G8 section-volume/flux and full retained-material oracle.

Coordinates are original-unit orthonormal (u,x,d). Native mass and Truck
geometry are not inputs to the expected volume or point classifications.
"""
import math,json,pathlib,subprocess,argparse
def leggauss(n):
 nodes=[];weights=[]
 for i in range(1,n+1):
  z=math.cos(math.pi*(i-.25)/(n+.5))
  for _ in range(16):
   a,b=1.,z
   for k in range(2,n+1):a,b=b,((2*k-1)*z*b-(k-1)*a)/k
   dp=n*(z*b-a)/(z*z-1);step=b/dp;z-=step
   if abs(step)<2e-16:break
  nodes.append(z);weights.append(2/((1-z*z)*dp*dp))
 return nodes,weights
K=math.sqrt(26); L=2*K; c=5/K; m=.2
X=lambda t:6*t+4*t**3
F=lambda t:c*(30*t-42*t*t+12*t**3)
DX=lambda t:6+12*t*t
DF=lambda t:c*(30-84*t+36*t*t)
top=(7-math.sqrt(19))/6; D=F(top)
def bisect(fn,lo,hi):
 for _ in range(60):
  mid=(lo+hi)/2
  if fn(mid)>0:hi=mid
  else:lo=mid
 return (lo+hi)/2

def geom(r):
 t=bisect(lambda t:F(t)-r-r/math.sqrt(1+(DF(t)/DX(t))**2),0,top)
 p=DF(t)/DX(t); q=math.hypot(1,p)
 return t,F(t),X(t)+r*p/q,L+r*(m-math.hypot(1,m)),r*(1-m/math.hypot(1,m))
def section(d,r,g):
 t,dt,xc,U,dc=g
 tl=bisect(lambda t:F(t)-d,0,top); tr=bisect(lambda t:d-F(t),top,1)
 a=X(tl); ap=DX(tl)/DF(tl); b=X(tr); bp=DX(tr)/DF(tr)
 h=L+m*d;hp=m
 if d<dt:v=math.sqrt(d*(2*r-d));a=xc-v;ap=(d-r)/v
 if d<dc:v=math.sqrt(d*(2*r-d));h=U+v;hp=(r-d)/v
 return a,b,h,ap,bp,hp

def calc(r,n):
 g=geom(r);t,dt,xc,U,dc=g; q,w=leggauss(n); area=[0.,0.,0.];flux=vol=0.;checks=[]
 for low,high in zip([0,dc,dt],[dc,dt,D]):
  for z,weight in zip([(x+1)*math.pi/4 for x in q],[x*math.pi/4 for x in w]):
   d=low+(high-low)*math.sin(z)**2; wt=weight*(high-low)*math.sin(2*z)
   a,b,h,ap,bp,hp=section(d,r,g);u=m*d; W=b-a;H=h-u
   normals=[[-W,0,m*W],[W,0,-hp*W],[0,-H,ap*H],[0,H,-bp*H]]
   points=[[u,(a+b)/2,d],[h,(a+b)/2,d],[(u+h)/2,a,d],[(u+h)/2,b,d]]
   for N,P in zip(normals,points):
    area=[a+wt*b for a,b in zip(area,N)];flux+=wt*sum(a*b for a,b in zip(N,P))/3
   vol+=wt*W*H
   if n==32:
    for N,P in zip(normals,points):
     norm=[a/math.sqrt(sum(b*b for b in N)) for a in N]
     for sign in [-1,1]:
      point=[a+sign*1e-4*b for a,b in zip(P,norm)]
      checks.append(dict(frame=point,inside=sign<0))
 area[2]-=(10-xc)*U
 return dict(r=r,n=n,t=t,dT=dt,dC=dc,X=xc,U=U,V=vol,flux=flux,area=area,removed=372-vol),checks

def inside(point,r=.5):
 u,x,d=point
 if not 0<d<D:return False
 a,b,h,*_=section(d,r,geom(r))
 return a<x<b and m*d<u<h

if __name__ == '__main__':
 parser=argparse.ArgumentParser()
 parser.add_argument('--step')
 parser.add_argument('--classifier',help='compiled p9_material.cpp STEP classifier')
 parser.add_argument('--scale',type=float,default=10.)
 args=parser.parse_args()
 for r in [.0001,.25,.5,.75]:
  rows=[calc(r,n)[0] for n in [16,32,48]]
  assert max(abs(x['V']-x['flux']) for x in rows)<1e-8
  assert max(abs(v) for x in rows for v in x['area'])<1e-8
  assert abs(rows[-1]['V']-rows[-2]['V'])<1e-8
  print(json.dumps(rows))
 witnesses=json.loads(pathlib.Path(__file__).with_name('g8_witnesses.json').read_text())['witnesses']
 assert len(witnesses)==770
 assert all(inside(w['frame'])==w['inside'] for w in witnesses)
 if args.step:
  if not args.classifier:parser.error('--step requires --classifier')
  points=''.join(f'{args.scale*x} {args.scale*(u+5*d)/K} {args.scale*(5*u-d)/K}\n' for w in witnesses for u,x,d in [w['frame']])
  result=subprocess.run([args.classifier,args.step],input=points,capture_output=True,text=True,check=True)
  states=[int(line.split()[1]) for line in result.stdout.splitlines() if line.startswith('STATE ')]
  assert len(states)==770
  failures=[i for i,(s,w) in enumerate(zip(states,witnesses)) if s!=(0 if w['inside'] else 1)]
  print(json.dumps(dict(material_checks=len(states),failures=failures)))
  assert not failures
