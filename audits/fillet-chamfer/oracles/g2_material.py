#!/usr/bin/env python3
"""Independent G2 analytic boundary-flux volume; no CAD/native/mesh inputs.
Coordinates are the original quadratic extrusion scaled by ten.
Gauss-Legendre integration; root solve is independent scalar bisection.
"""
import json, math, time
from pathlib import Path
import numpy as np
from numpy.polynomial.legendre import leggauss
A=np.array([200.,0.,1000.]); n=np.array([1.,0.,0.]); m=np.array([-5.,0.,1.])/math.sqrt(26)
D=np.cross(n,m);D/=np.linalg.norm(D)
SOURCE=400000000/3

def source_margins(X):
    return [float(np.min(X[...,0])),float(np.min(200-X[...,0])),float(np.min(X[...,2]-5*X[...,0]-X[...,1]**2/2000)),float(np.min(500+5*X[...,0]-X[...,2]))]

def gauss(order):
    x,w=leggauss(order);return (x+1)/2,w/2

def frame(s,r):
    s=np.asarray(s);L=np.sqrt(26+s*s);K=np.sqrt(1+s*s)
    zero=np.zeros_like(s)
    B=np.stack([zero,1000*s,500*s*s],axis=-1)
    N=np.stack([5/L,s/L,-1/L],axis=-1)
    V=np.stack([zero,s/K,-1/K],axis=-1)
    C=np.stack([zero+200-r,1000*s-r*s/L,500*s*s+1000-5*r+26*r/L],axis=-1)
    # Derivatives with respect to s, NOT original u=(s+1)/2.
    Cp=np.stack([zero,1000-26*r/L**3,1000*s-26*r*s/L**3],axis=-1)
    Vp=np.stack([zero,1/K**3,s/K**3],axis=-1)
    return B,N,V,C,Cp,Vp,np.arccos(5/L)

def bisect(fn,lo,hi):
    lo=np.asarray(lo,dtype=float);hi=np.asarray(hi,dtype=float)
    flo=fn(lo);fhi=fn(hi)
    assert np.all(np.isfinite(flo)) and np.all(np.isfinite(fhi))
    assert np.all(flo*fhi<=0), (flo,fhi)
    for _ in range(60):
        mid=(lo+hi)/2;fm=fn(mid);left=flo*fm<=0
        hi=np.where(left,mid,hi);lo=np.where(left,lo,mid);flo=np.where(left,flo,fm)
    return (lo+hi)/2

def seam(s,q,r):
    _,_,V,C,_,_,alpha=frame(s,r)
    return C[...,2]-(1500-5*r-r*math.sqrt(26))+r*np.sin(q*alpha)*(V[...,2]-1)

def volume(r,order):
    x,w=gauss(order)
    # Reflection symmetry gives the selected positive station from remote root.
    sp=float(bisect(lambda s:seam(s,0,r),-1.,0.));ss=-sp
    sq=float(bisect(lambda s:seam(s,1,r),-1.,0.))
    # Retained E: D.(B_s cross A) * D.B * v = 2e8*s^2*v.
    s=sq+(ss-sq)*x
    _,N,_,_,_,_,_=frame(s,r)
    vmax=(200-r+r*N[:,0])/200
    extrusion_contact=float(np.dot(w,2e8*s*s*vmax)*(ss-sq))
    contact_area=float(np.dot(w,-200000*s*vmax)*(ss-sq))
    # Cylinder-clipped remote E, s=-1+(sq+1)t^2 removes endpoint sqrt.
    s=-1+(sq+1)*x*x;K=500*(s*s-1)+r*math.sqrt(26)
    discr=26*r*r-K*K
    assert np.all(discr>=-1e-8)
    height=200-r+(-5*K+np.sqrt(np.maximum(discr,0)))/26
    extrusion_remote=float(np.dot(w,2e8*s*s*(height/200)*2*(sq+1)*x))
    remote_area=float(np.dot(w,-200000*s*(height/200)*2*(sq+1)*x))
    # Canal parameter q=phi/alpha(s), s in[remote seam(q), selected station].
    sr=bisect(lambda s:seam(s,x,r),-np.ones_like(x),np.zeros_like(x))
    s=sr[:,None]+(ss-sr[:,None])*x[None,:]
    _,_,V,C,Cp,Vp,alpha=frame(s,r);phi=x[:,None]*alpha
    radial=-n*np.sin(phi)[...,None]+V*np.cos(phi)[...,None]
    Xu=Cp+r*Vp*np.sin(phi)[...,None]
    Xq=r*alpha[...,None]*radial
    X=C+r*(n*np.cos(phi)[...,None]+V*np.sin(phi)[...,None])
    # Outward normal equals X_q cross X_s. alpha' term cancels exactly.
    area=np.cross(Xq,Xu)
    canal=float(np.sum(w[:,None]*w[None,:]*(ss-sr[:,None])*(X@D)*(area@D)))
    canal_area=float(np.sum(w[:,None]*w[None,:]*(ss-sr[:,None])*(area@D)))
    margins=source_margins(X)
    # Selected longitudinal cylinder, analytically integrate its A station.
    _,Ns,_,C0,_,_,_=frame(ss,r)
    gamma=math.acos(float(Ns@m));T=(m-Ns*math.cos(gamma))/math.sin(gamma)
    theta=gamma*x;R=Ns*np.cos(theta)[:,None]+T*np.sin(theta)[:,None]
    Rp=gamma*(-Ns*np.sin(theta)[:,None]+T*np.cos(theta)[:,None])
    # Select outward sign from analytic normal at the arc midpoint.
    oriented=np.cross(A,r*Rp);alignment=np.sum(oriented*R,axis=1)
    assert np.all(alignment*alignment[0]>0)
    oriented*=math.copysign(1.,alignment[0])
    lower=(-200+r-r*(R@n))/float(n@A)
    cylinder=float(np.dot(w,(oriented@D)*((C0+r*R)@D)*(-lower))) # D.A=0
    cylinder_area=float(np.dot(w,(oriented@D)*(-lower)))
    cylinder_points=C0+r*R[:,None,:]+(lower[:,None]*(1-x[None,:]))[...,None]*A
    cylinder_margins=source_margins(cylinder_points)
    assert min(cylinder_margins)>-1e-8, cylinder_margins
    # Selected sphere: normalized barycentric spherical normal triangle.
    u=x[:,None];v=x[None,:]
    b=(1-u)[...,None]*(1-v)[...,None]*n+u[...,None]*m+(1-u)[...,None]*v[...,None]*Ns
    length=np.linalg.norm(b,axis=-1);R=b/length[...,None]
    sphere_margins=source_margins(C0+r*R)
    assert min(sphere_margins)>-1e-8, sphere_margins
    det=abs(float(np.linalg.det(np.stack([n,m,Ns]))))
    jac=det*(1-u)/length**3
    sphere=float(np.sum(w[:,None]*w[None,:]*r*r*((C0+r*R)@D)*(R@D)*jac))
    numerical_sphere=sphere
    # Independent exact spherical moments from boundary great-circle arcs.
    vertices=[n,Ns,m];signed_det=float(np.linalg.det(np.stack(vertices)));assert signed_det>0
    solid_angle=2*math.atan2(signed_det,1+vertices[0]@vertices[1]+vertices[1]@vertices[2]+vertices[2]@vertices[0])
    first=np.zeros(3);second=solid_angle/3
    for a,b in zip(vertices,vertices[1:]+vertices[:1]):
        angle=math.acos(float(a@b));edge=np.cross(a,b)/math.sin(angle)
        first+=.5*angle*edge
        second+=(edge@D)*((a+b)@D)*math.tan(angle/2)/3
    sphere=r*r*(C0@D)*(first@D)+r**3*second
    sphere_area=r*r*(first@D)
    area_residual=contact_area+remote_area+canal_area+cylinder_area+sphere_area
    assert abs(area_residual)<1e-7, area_residual
    assert min(margins)>-1e-8, margins
    values=dict(extrusion_contact=extrusion_contact,extrusion_remote=extrusion_remote,canal=canal,cylinder=cylinder,sphere=sphere)
    total=sum(values.values())
    assert math.isfinite(total) and 0<total<SOURCE
    return dict(radius=r,order=order,source_volume=SOURCE,volume=total,removed_volume=SOURCE-total,patch_fluxes=values,stations=dict(remote_center=sp,selected=ss,remote_E_contact=sq),sphere_min_sampled_b_length=float(length.min()),numerical_sphere_flux=numerical_sphere,sphere_quadrature_error=numerical_sphere-sphere,area_flux_residual=area_residual,canal_min_source_margins=margins,cylinder_min_source_margins=cylinder_margins,sphere_min_source_margins=sphere_margins)

if __name__=='__main__':
    results=[]
    for r in [.25,.5,1.,3.,5.,6.,10.]:
        for order in [32,64,128,256]:
            start=time.monotonic();result=volume(r,order);result['seconds']=time.monotonic()-start
            results.append(result);print(json.dumps(result),flush=True)
    highest=[p for p in results if p['order']==256]
    assert all(a['volume']>b['volume'] for a,b in zip(highest,highest[1:]))
    Path('target/b-g2-material-oracle.json').write_text(json.dumps(results,indent=2)+'\n')
