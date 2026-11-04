"""Independent G4 section volume, flux and material witnesses; no native mass input."""
import math, json, pathlib, argparse
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
def bisect(fn,lo,hi):
 for _ in range(60):
  mid=(lo+hi)/2
  if fn(mid)>0:hi=mid
  else:lo=mid
 return (lo+hi)/2
sqrt=math.sqrt
L=2*sqrt(26);alpha=sqrt(26/819);beta=-sqrt(26/126)/5;k=sqrt(1+alpha*alpha);N=sqrt(k*k+beta*beta)
XC=[v/sqrt(819) for v in [0,260,385,575,630]];FC=[v/sqrt(126) for v in [0,200,175,200,0]]
def bez(c,t):
 while len(c)>1:c=[a*(1-t)+b*t for a,b in zip(c,c[1:])]
 return c[0]
def der(c):return [(b-a)*(len(c)-1) for a,b in zip(c,c[1:])]
X=lambda t:bez(XC,t);F=lambda t:bez(FC,t);DX=lambda t:bez(der(XC),t);DF=lambda t:bez(der(FC),t)
peak=bisect(lambda t:-DF(t),0,1);D=F(peak)
def geom(r):
 t=bisect(lambda t:F(t)-r-r/math.hypot(1,DF(t)/DX(t)),0,peak);p=DF(t)/DX(t);q=math.hypot(1,p)
 return t,F(t),X(t)+r*p/q,L+r*(beta-N),r*(1-beta/N)
def section(d,r):
 t,dt,xc,C,dc=geom(r);tl=bisect(lambda t:F(t)-d,0,peak);tr=bisect(lambda t:d-F(t),peak,1)
 a,b=X(tl),X(tr);ap,bp=DX(tl)/DF(tl),DX(tr)/DF(tr);h=L+beta*d;hp=beta
 if d<dt:v=sqrt(d*(2*r-d));a=xc-v;ap=(d-r)/v
 if d<dc:v=sqrt(d*(2*r-d));h=C+k*v;hp=k*(r-d)/v
 return a,b,h,ap,bp,hp

def inside(u,x,d,r=1):
 if not 0<d<D:return False
 a,b,h,*_=section(d,r);return a<x<b and alpha*x+beta*d<u<alpha*x+h

def source_inside(u,x,d):
 if not 0<d<D:return False
 left=bisect(lambda t:F(t)-d,0,peak)
 right=bisect(lambda t:d-F(t),peak,1)
 return X(left)<x<X(right) and alpha*x+beta*d<u<alpha*x+beta*d+L

def calc(r,n):
 t,dt,xc,C,dc=geom(r);q,w=leggauss(n);vol=flux=0.;area=[0.,0.,0.];witness=[]
 for low,high in zip([0,dc,dt],[dc,dt,D]):
  for z,wt in zip([(v+1)*math.pi/4 for v in q],[v*math.pi/4 for v in w]):
   d=low+(high-low)*math.sin(z)**2;wt*= (high-low)*math.sin(2*z)
   a,b,h,ap,bp,hp=section(d,r);W=b-a;H=h-beta*d;x=(a+b)/2
   ns=[[-W,alpha*W,beta*W],[W,-alpha*W,-hp*W],[0,-H,ap*H],[0,H,-bp*H]]
   ps=[[alpha*x+beta*d,x,d],[alpha*x+h,x,d],[alpha*a+(beta*d+h)/2,a,d],[alpha*b+(beta*d+h)/2,b,d]]
   vol+=wt*W*H
   for P,A in zip(ps,ns):
    area=[v+wt*j for v,j in zip(area,A)];flux+=wt*sum(v*j for v,j in zip(P,A))/3
    if n==32 and r==1:
     norm=sqrt(sum(v*v for v in A))
     for eps in [-1e-4,1e-4]:
      point=[v+eps*j/norm for v,j in zip(P,A)];witness.append([*point,int(inside(*point))])
 area[2]-=(X(1)-xc)*C
 return dict(r=r,n=n,V=vol,flux=flux,area=area,t=t,dT=dt,dC=dc,xc=xc,C=C),witness

def main():
 parser=argparse.ArgumentParser()
 parser.add_argument('--output',type=pathlib.Path,required=True)
 args=parser.parse_args()
 rows=[]; witnesses=[]
 for r in [.0001,.5,1,1.5]:
  for n in [16,32,48]:
   row,w=calc(r,n);rows.append(row)
   if w:witnesses=w
 for sign in [-1,1]:witnesses.append([L/2,(geom(1)[2]+X(1))/2,sign*1e-4,int(sign>0)])
 for d in [.05,.4,.9,1.,1.04,1.08,1.10,1.25,1.46]:
  a,b,h,*_=section(d,1)
  for dx in [-1e-4,1e-4]:
   for du in [-1e-4,1e-4]:
    x=a+dx;u=alpha*x+h+du
    witnesses.append([u,x,d,int(inside(u,x,d))])
 assert len(witnesses)==806
 for r in [.0001,.5,1,1.5]:
  group=[row for row in rows if row['r']==r]
  assert max(abs(row['V']-row['flux']) for row in group)<1e-8
  assert abs(group[-1]['V']-group[-2]['V'])<1e-8
  assert max(abs(v) for row in group for v in row['area'])<1e-8
 args.output.write_text(json.dumps(dict(frame=dict(L=L,alpha=alpha,beta=beta,k=k,N=N),results=rows,witnesses=[dict(frame=w[:3],inside=bool(w[3]),source_inside=source_inside(*w[:3])) for w in witnesses]),indent=2)+'\n')
if __name__=='__main__':main()
