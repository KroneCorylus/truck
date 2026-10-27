#!/usr/bin/env python3
"""Independent quartic G6 volume from boundary flux, in source-script units.

No CAD or OCCT inputs. F(X) = D (D dot X), with unit D perpendicular
both to cap and closing wall normals, has divergence one. Planar caps and
wall therefore contribute zero. Integrate the retained extrusion, canal,
longitudinal cylinder and spherical corner. The canal integration is split
at its wall contact: cylinder seam first, then canal/wall continuation.
Area-flux closure is an independent check on signs and integration domains.
"""
import numpy as np, math
from numpy.polynomial.legendre import leggauss
A=np.array([0.,2.,10.]); n=np.array([0.,0.,1.]); m=np.array([-10.,5.,-1.])/math.sqrt(126)
control_x=25.
D=np.cross(n,m);D/=np.linalg.norm(D);w=(m-n*m[2]);w/=np.linalg.norm(w);beta=math.acos(m[2])
def frame(t,r):
 t=np.asarray(t);z=np.zeros_like(t)
 B=np.stack([80*t-120*t**2+100*t**3-50*t**4,30*t**2-20*t**3+10*t**4,z],-1)
 Bp=np.stack([80-240*t+300*t**2-200*t**3,60*t-60*t**2+40*t**3,z],-1)
 delta=control_x-25.
 B[...,0]+=4*delta*t**3*(1-t)
 Bp[...,0]+=4*delta*(3*t**2-4*t**3)
 Bpp=np.stack([-240+600*t-600*t**2,60-120*t+120*t**2,z],-1)
 Bpp[...,0]+=4*delta*(6*t-12*t**2)
 W=np.cross(Bp,A);Wp=np.cross(Bpp,A);L=np.linalg.norm(W,axis=-1);N=W/L[...,None]
 Np=(Wp-N*np.sum(N*Wp,axis=-1)[...,None])/L[...,None]
 V=N-n*N[...,2,None];Vp=Np-n*Np[...,2,None];L=np.linalg.norm(V,axis=-1);V=V/L[...,None];Vp=(Vp-V*np.sum(V*Vp,axis=-1)[...,None])/L[...,None]
 v=(10-r+r*N[...,2])/10; C=B+v[...,None]*A-r*N;Cp=Bp+(r*Np[...,2]/10)[...,None]*A-r*Np
 return B,Bp,N,V,C,Cp,Vp,np.arccos(N[...,2]),v

def root(fn,lo,hi):
 lo=np.asarray(lo,dtype=float);hi=np.asarray(hi,dtype=float);flo=fn(lo);fhi=fn(hi)
 assert np.all(flo*fhi<=1e-12),(flo,fhi)
 for _ in range(60):
  mid=(lo+hi)/2;fm=fn(mid);left=flo*fm<=0
  hi=np.where(left,mid,hi);lo=np.where(left,lo,mid);flo=np.where(left,flo,fm)
 return (lo+hi)/2

def volume(r,order):
 x,wg=leggauss(order);x=(x+1)/2;wg/=2
 cline=(-r-(10-r)*m[2])/float(m@w)
 def seam(t,phi):
  _,_,_,V,C,_,_,_,_=frame(t,r)
  return C@w-cline+r*np.sin(phi)*(V@w-1)
 ts=float(root(lambda t:frame(t,r)[4]@m+r,0.,.5))
 tw=float(root(lambda t:seam(t,beta),.9,1.1));qc=beta/float(frame(tw,r)[7])
 def terminal(t,q,wall):
  _,_,_,V,C,_,_,alpha,_=frame(t,r);phi=q*alpha
  if not wall:return seam(t,phi)
  return (C+r*(n*np.cos(phi)[...,None]+V*np.sin(phi)[...,None]))@m
 flux=[];area=[]
 t=ts+(1-ts)*x;B,Bp,N,V,C,Cp,Vp,alpha,v=frame(t,r)
 Earea=np.cross(Bp,A)@D
 flux.append(float(np.dot(wg,Earea*((B@D)*v+(A@D)*v*v/2))*(1-ts)))
 area.append(float(np.dot(wg,Earea*v)*(1-ts)))
 for qlo,qhi,wall in [(0.,qc,False),(qc,1.,True)]:
  q=qlo+(qhi-qlo)*x;tr=root(lambda t:terminal(t,q,wall),np.full_like(q,.5),np.full_like(q,1.1))
  t=ts+(tr[:,None]-ts)*x[None,:]
  _,_,_,V,C,Cp,Vp,alpha,_=frame(t,r);phi=q[:,None]*alpha
  X=C+r*(n*np.cos(phi)[...,None]+V*np.sin(phi)[...,None])
  Xt=Cp+r*Vp*np.sin(phi)[...,None]
  Xq=r*alpha[...,None]*(-n*np.sin(phi)[...,None]+V*np.cos(phi)[...,None])
  oriented=np.cross(Xq,Xt)@D;weights=wg[:,None]*wg[None,:]*(tr[:,None]-ts)*(qhi-qlo)
  flux.append(float(np.sum(weights*(X@D)*oriented)));area.append(float(np.sum(weights*oriented)))
 _,_,Ns,_,C0,_,_,_,_=frame(ts,r);gamma=math.acos(float(Ns@m));T=(m-Ns*math.cos(gamma))/math.sin(gamma)
 theta=gamma*x;R=Ns*np.cos(theta)[:,None]+T*np.sin(theta)[:,None];Rp=gamma*(-Ns*np.sin(theta)[:,None]+T*np.cos(theta)[:,None])
 oriented=np.cross(A,r*Rp);oriented*=math.copysign(1.,float(np.sum(oriented[0]*R[0])))
 lower=-(C0[2]+r*R[:,2])/10
 flux.append(float(np.dot(wg,(oriented@D)*(((C0+r*R)@D)*(-lower)-(A@D)*lower*lower/2))))
 area.append(float(np.dot(wg,(oriented@D)*(-lower))))
 u=x[:,None];v=x[None,:];b=(1-u)[...,None]*(1-v)[...,None]*n+u[...,None]*m+(1-u)[...,None]*v[...,None]*Ns
 L=np.linalg.norm(b,axis=-1);R=b/L[...,None];jac=abs(float(np.linalg.det(np.stack([n,m,Ns]))))*(1-u)/L**3
 flux.append(float(np.sum(wg[:,None]*wg[None,:]*r*r*((C0+r*R)@D)*(R@D)*jac)))
 area.append(float(np.sum(wg[:,None]*wg[None,:]*r*r*(R@D)*jac)))
 print(r,order,'volume',sum(flux),'area residual',sum(area),'flux',flux,flush=True)
 assert abs(sum(area))<1e-7
 return sum(flux)
if __name__ == '__main__':
 for r in [.5,1.,1.2,4.884,4.8842]:
  values = [volume(r,order) for order in [32,64,128]]
  assert max(values)-min(values) < 1e-8
  assert 0 < values[-1] < 2300

 control_x=24.9
 values=[volume(1.,order) for order in [64,128]]
 assert max(values)-min(values)<1e-8
