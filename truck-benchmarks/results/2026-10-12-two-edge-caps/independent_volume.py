from math import sqrt,sin,cos,atan,asin,acos

def quad(f,a,b,tol=1e-11):
    fa,fb,fm=f(a),f(b),f((a+b)/2)
    def rec(a,b,fa,fb,fm,s,tol,n):
        m=(a+b)/2; fl=f((a+m)/2); fr=f((m+b)/2)
        l=(m-a)*(fa+4*fl+fm)/6; r=(b-m)*(fm+4*fr+fb)/6
        if n==0 or abs(l+r-s)<=15*tol: return l+r+(l+r-s)/15
        return rec(a,m,fa,fm,fl,l,tol/2,n-1)+rec(m,b,fm,fb,fr,r,tol/2,n-1)
    return rec(a,b,fa,fb,fm,(b-a)*(fa+4*fm+fb)/6,tol,22)
def root(f,a,b):
    for _ in range(70):
        m=(a+b)/2
        if f(a)*f(m)<=0: b=m
        else: a=m
    return (a+b)/2
qmax=root(lambda q:q*(1-1/sqrt(26+q*q))-1,1,2)
def inner(q):
    L=sqrt(26+q*q); M=sqrt(1+q*q)
    cy=q*(1-1/L); cz=q*q/2-4+26/L; dcy=1-1/L+q*q/L**3
    b=atan(M/5) if q<=1 else asin(max(0,min(1,(1-cy)*M/q)))
    def f(g):
        x=-.8+cos(g); y=cy+q/M*sin(g); z=cz-sin(g)/M
        delta=z-y*y/2-5*x; height=.5-y*y/2
        return max(0,min(delta,height))*sin(g)*(dcy+sin(g)/M**3)
    splits=[0,b]; D=cz+3.5; A=sqrt(25+1/M**2)
    if -1<D/A<1:
        d=atan(1/(5*M)); h=acos(D/A)
        splits+= [g for g in [d-h,d+h] if 0<g<b]
    splits.sort()
    return sum(quad(f,a,b,1e-13) for a,b in zip(splits,splits[1:]))
qcap=root(lambda q:q*q/2-4+26/sqrt(26+q*q)+3.5-5,.5,1.2)
qtangent=root(lambda q:q*q/2-4+26/sqrt(26+q*q)+3.5-sqrt(25+1/(1+q*q)),1,qmax)
breaks=[0,qcap,1,qtangent,qmax]
print('breaks',breaks)
for tol in [1e-9,1e-11,1e-13]:
    removed=2*sum(quad(inner,a,b,tol) for a,b in zip(breaks,breaks[1:]))
    print('tol',tol,'removed',removed,'volume',(2/15-removed)*1e9,flush=True)
