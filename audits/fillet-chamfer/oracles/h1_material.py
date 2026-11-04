"""Independent H1 rolling-ball flux and material oracle (NumPy only).

Polynomial source, plane/canal equations and Gaussian quadrature do not call
Truck or OCCT. Volumes are converged numerical integrals, not exact values.
The printed inverse-monotone-x containment checks are sampled witnesses,
not a global certificate. Native discrepancies remain unclassified.
"""
import numpy as np, math, json
from numpy.polynomial.legendre import leggauss
A=np.array([0.,2.,10.]); n=np.array([0.,0.,1.]);m=np.array([0.,-10.,2.])/math.sqrt(104);D=np.array([1.,0.,0.]);w=np.array([0.,-1.,0.]);beta=math.acos(m[2])
def frame(t,r):
 t=np.asarray(t);z=np.zeros_like(t)
 B=np.stack([6*t+4*t**3,30*t-42*t*t+12*t**3,z],-1)
 Bp=np.stack([6+12*t*t,30-84*t+36*t*t,z],-1)
 Bpp=np.stack([24*t,-84+72*t,z],-1)
 W=np.cross(A,Bp);Wp=np.cross(A,Bpp);L=np.linalg.norm(W,axis=-1);N=W/L[...,None];Np=(Wp-N*np.sum(N*Wp,axis=-1)[...,None])/L[...,None]
 V=N-n*N[...,2,None];Vp=Np-n*Np[...,2,None];L=np.linalg.norm(V,axis=-1);V=V/L[...,None];Vp=(Vp-V*np.sum(V*Vp,axis=-1)[...,None])/L[...,None]
 v=(10-r+r*N[...,2])/10;C=B+v[...,None]*A-r*N;Cp=Bp+(r*Np[...,2]/10)[...,None]*A-r*Np
 return B,Bp,N,V,C,Cp,Vp,np.arccos(N[...,2]),v

def root(fn,lo,hi):
 lo=np.asarray(lo,dtype=float);hi=np.asarray(hi,dtype=float);fl=fn(lo);fh=fn(hi)
 assert np.all(fl*fh<=0),(fl,fh)
 for _ in range(60):
  mid=(lo+hi)/2;fm=fn(mid);left=fl*fm<=0;hi=np.where(left,mid,hi);lo=np.where(left,lo,mid);fl=np.where(left,fl,fm)
 return (lo+hi)/2

def calc(r,order):
 x,wg=leggauss(order);x=(x+1)/2;wg/=2
 cline=(-r-(10-r)*m[2])/float(m@w)
 def seam(t,phi):
  _,_,_,V,C,*_=frame(t,r)
  return C@w-cline+r*np.sin(phi)*(V@w-1)
 ts=float(root(lambda t:frame(t,r)[4]@m+r,.5,1.))
 tp=float(root(lambda t:seam(t,0.),0.,.5));tw=float(root(lambda t:seam(t,beta),-.1,.5));qc=beta/float(frame(tw,r)[7])
 def terminal(t,q,wall):
  _,_,_,V,C,_,_,alpha,_=frame(t,r);phi=q*alpha
  return ((C+r*(n*np.cos(phi)[...,None]+V*np.sin(phi)[...,None]))@m if wall else seam(t,phi))
 flux=[];area=[]
 t=ts*x;B,Bp,N,V,C,Cp,Vp,alpha,v=frame(t,r);E=np.cross(A,Bp)@D
 flux.append(float(np.dot(wg,E*(B@D)*v)*ts));area.append(float(np.dot(wg,E*v)*ts))
 for qlo,qhi,wall in [(0.,qc,False),(qc,1.,True)]:
  q=qlo+(qhi-qlo)*x;tr=root(lambda t:terminal(t,q,wall),np.full_like(q,-.1),np.full_like(q,.5))
  t=tr[:,None]+(ts-tr[:,None])*x[None,:]
  _,_,_,V,C,Cp,Vp,alpha,_=frame(t,r);phi=q[:,None]*alpha
  X=C+r*(n*np.cos(phi)[...,None]+V*np.sin(phi)[...,None]);Xt=Cp+r*Vp*np.sin(phi)[...,None];Xq=r*alpha[...,None]*(-n*np.sin(phi)[...,None]+V*np.cos(phi)[...,None]);ori=np.cross(Xt,Xq)@D;weight=wg[:,None]*wg[None,:]*(ts-tr[:,None])*(qhi-qlo)
  flux.append(float(np.sum(weight*(X@D)*ori)));area.append(float(np.sum(weight*ori)))
 _,_,Ns,_,C0,_,_,_,_=frame(ts,r);gamma=math.acos(float(Ns@m));T=(m-Ns*math.cos(gamma))/math.sin(gamma)
 theta=gamma*x;R=Ns*np.cos(theta)[:,None]+T*np.sin(theta)[:,None];Rp=gamma*(-Ns*np.sin(theta)[:,None]+T*np.cos(theta)[:,None]);ori=np.cross(A,r*Rp);ori*=math.copysign(1.,float(np.sum(ori[0]*R[0])));lower=-(C0[2]+r*R[:,2])/10
 flux.append(float(np.dot(wg,(ori@D)*((C0+r*R)@D)*(-lower))));area.append(float(np.dot(wg,(ori@D)*(-lower))))
 u=x[:,None];v=x[None,:];b=(1-u)[...,None]*(1-v)[...,None]*n+u[...,None]*m+(1-u)[...,None]*v[...,None]*Ns;L=np.linalg.norm(b,axis=-1);R=b/L[...,None];jac=abs(float(np.linalg.det(np.stack([n,m,Ns]))))*(1-u)/L**3
 flux.append(float(np.sum(wg[:,None]*wg[None,:]*r*r*((C0+r*R)@D)*(R@D)*jac)));area.append(float(np.sum(wg[:,None]*wg[None,:]*r*r*(R@D)*jac)))
 print(json.dumps(dict(radius=r,order=order,volume=sum(flux),area_residual=sum(area),tp=tp,tw=tw,ts=ts,beta=beta,alpha0=float(frame(0,r)[7]),alpha1=float(frame(1,r)[7]),flux=flux)))
 if r==.5 and order==64:
  for name,t,phi in [('P',tp,0),('Q',tw,beta),('R',0,float(frame(0,r)[7]))]:
   f=frame(t,r);print(name,(f[4]+r*(n*np.cos(phi)+f[3]*np.sin(phi))).tolist())
for r in [.01,.3,.5,.6,1.]:
 for order in [32,64,128]:calc(r,order)
# Signed planar curvature numerator: 1008*t*t - 288*t - 504.
r=.5
q=np.linspace(0,1,1001);beta=math.acos(m[2]);tw=float(root(lambda t: (frame(t,r)[4]+r*(n*math.cos(beta)+frame(t,r)[3]*math.sin(beta)))@m,-.1,.5));qc=beta/float(frame(tw,r)[7]);q=np.linspace(qc,1,1001)
def wallfun(t):
 f=frame(t,r);phi=q*f[7];return (f[4]+r*(n*np.cos(phi)[...,None]+f[3]*np.sin(phi)[...,None]))@m
tr=root(wallfun,np.full_like(q,-.1),np.full_like(q,.1));print('continuation t range',tr.min(),tr.max(),'at q',q[tr.argmin()]);print('correct inflection',np.roots([1008,-288,-504]))
# Source containment by exact monotone x inverse; no tangent-halfspace convexity assumption.
def inside(X):
 X=np.asarray(X);tx=root(lambda t:6*t+4*t**3-X[...,0],np.zeros(X.shape[:-1]),np.ones(X.shape[:-1]));y=X[...,1]-.2*X[...,2];return min(float(X[...,2].min()),float((10-X[...,2]).min()),float(y.min()),float((30*tx-42*tx*tx+12*tx**3-y).min()))
print('continuation source margin',inside(frame(tr,r)[4]+r*(n*np.cos(q*frame(tr,r)[7])[...,None]+frame(tr,r)[3]*np.sin(q*frame(tr,r)[7])[...,None])))
ts=float(root(lambda t:frame(t,r)[4]@m+r,.5,1.));qc=beta/float(frame(tw,r)[7]);sample=[]
for wall,qlo,qhi in [(False,0,qc),(True,qc,1)]:
 q=np.linspace(qlo,qhi,129)
 def terminal(t):
  f=frame(t,r);phi=q*f[7]
  if wall:return (f[4]+r*(n*np.cos(phi)[...,None]+f[3]*np.sin(phi)[...,None]))@m
  cline=(-r-(10-r)*m[2])/float(m@w)
  return f[4]@w-cline+r*np.sin(phi)*(f[3]@w-1)
 tr=root(terminal,np.full_like(q,-.1),np.full_like(q,.5));t=tr[:,None]+(ts-tr[:,None])*np.linspace(0,1,129)[None,:];f=frame(t,r);phi=q[:,None]*f[7];X=f[4]+r*(n*np.cos(phi)[...,None]+f[3]*np.sin(phi)[...,None]);sample.append(X.reshape(-1,3))
f=frame(ts,r);Ns=f[2];C0=f[4];u=np.linspace(0,1,129)[:,None];v=np.linspace(0,1,129)[None,:];b=(1-u)[...,None]*(1-v)[...,None]*n+u[...,None]*m+(1-u)[...,None]*v[...,None]*Ns;R=b/np.linalg.norm(b,axis=-1)[...,None];sample.append((C0+r*R).reshape(-1,3))
gamma=math.acos(float(Ns@m));T=(m-Ns*math.cos(gamma))/math.sin(gamma);theta=np.linspace(0,gamma,129);R=Ns*np.cos(theta)[:,None]+T*np.sin(theta)[:,None];lo=-(C0[2]+r*R[:,2])/10;X=C0+r*R[:,None,:]+(lo[:,None]*(1-np.linspace(0,1,129))[None,:])[...,None]*A;sample.append(X.reshape(-1,3))
print('sampled source margins canal1 canal2 sphere ruling-cylinder',[inside(X) for X in sample])
t=np.linspace(0,1,10001);Bp=frame(t,r)[1];Bpp=np.stack([24*t,-84+72*t,np.zeros_like(t)],-1);axis=A/np.linalg.norm(A);P=Bp-np.sum(Bp*axis,axis=-1)[...,None]*axis;PP=Bpp-np.sum(Bpp*axis,axis=-1)[...,None]*axis;k=np.linalg.norm(np.cross(P,PP),axis=-1)/np.linalg.norm(P,axis=-1)**3;print('sampled max r curvature',r*k.max(),'at',t[k.argmax()])

# Independent divided-difference clearance calculation in the plane normal to A.
# This bound covers all pairs, including the diagonal by polynomial continuation.
axis=A/np.linalg.norm(A)
project=lambda v: v-axis*np.dot(axis,v)
p1=project(np.array([6.,30.,0.]))
p2=project(np.array([0.,-42.,0.]))
p3=project(np.array([4.,12.,0.]))
limit=float('inf')
limiting_box=None
for i in range(32):
 t0,t1=i/32,(i+1)/32
 derivative=lambda t:p1+2*p2*t+3*p3*t*t
 ds=[derivative(t0),derivative(t0)+(t1-t0)*(p2+3*p3*t0),derivative(t1)]
 direction=ds[1]/np.linalg.norm(ds[1]);speed=min(np.dot(v,direction) for v in ds)
 for j in range(32):
  s0,s1=j/32,(j+1)/32;sm=(s0+s1)/2;tm=(t0+t1)/2
  direction=p1+p2*(sm+tm)+p3*(sm*sm+sm*tm+tm*tm);direction/=np.linalg.norm(direction)
  lower=lambda coefficient,lo,hi:coefficient*(lo if coefficient>=0 else hi)
  distance=np.dot(p1,direction)+lower(np.dot(p2,direction),s0+t0,s1+t1)+lower(np.dot(p3,direction),s0*s0+s0*t0+t0*t0,s1*s1+s1*t1+t1*t1)
  numerator=min(0.,*(np.dot(np.cross(axis,v),p2+p3*q) for v in ds for q in [s0+2*t0,s1+2*t1]))
  if numerator<0:
   bound=distance*distance*speed/(-2*numerator)
   if bound<limit:limit=bound;limiting_box=(i,j)
print('conservative global disk-clearance radius limit',limit,'parameter box',limiting_box)

# Bounded root-order witnesses: scan the full original parameter domain as well
# as the retained canal. Sign-change counts cannot exclude unsampled tangent roots.
def root_order_witnesses():
 r=.5
 cline=(-r-(10-r)*m[2])/float(m@w)
 ts=float(root(lambda t:frame(t,r)[4]@m+r,.8,1.))
 grid=np.linspace(0,1,16385)
 right=[]
 for phi in np.linspace(0,beta,65):
  def residual(t):
   f=frame(t,r)
   return f[4]@w-cline+r*np.sin(phi)*(f[3]@w-1)
  values=residual(grid)
  crossings=np.flatnonzero(values[:-1]*values[1:]<0)
  assert len(crossings)==2,(phi,crossings)
  roots=[float(root(residual,grid[i],grid[i+1])) for i in crossings]
  assert 0 < roots[0] < 1/16 < ts <= roots[1]+1e-14 <= 1+1e-14
  retained=np.linspace(roots[0],ts,4097)[1:-1]
  assert np.all(residual(retained)<0)
  right.append(roots[1])
 assert np.all(np.diff(right)>0)
 assert abs(right[0]-.950838009792891)<1e-14
 assert abs(right[-1]-.997943594311940)<1e-14
 for phi in np.linspace(beta,float(frame(0,r)[7]),65):
  def residual(t):
   f=frame(t,r)
   return (f[4]+r*(n*np.cos(phi)+f[3]*np.sin(phi)))@m
  left=0. if phi==float(frame(0,r)[7]) else float(root(residual,0.,1/16))
  assert np.all(residual(np.linspace(left,ts,4097)[1:-1])<0)
 print(json.dumps(dict(root_order='sampled pass',phi_sections=65,full_domain_stations=16385,
     retained_stations=4097,right_phi0=right[0],right_phibeta=right[-1],
     limits='No competing sign-changing branch on this grid; no exclusion of unsampled or tangent roots, no global proof.')))
root_order_witnesses()
