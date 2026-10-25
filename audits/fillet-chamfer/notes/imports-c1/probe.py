import ast,bisect,json,math,re,zipfile
from functools import lru_cache
from pathlib import Path
Z=zipfile.ZipFile('audits/fillet-chamfer/capture/2026-10-22-torus-corners/artifacts.zip')
def add(a,b):return [x+y for x,y in zip(a,b)]
def sub(a,b):return [x-y for x,y in zip(a,b)]
def mul(a,s):return [x*s for x in a]
def dot(a,b):return sum(x*y for x,y in zip(a,b))
def norm(a):return math.sqrt(dot(a,a))
def solve(a,b):
 a=[list(row)+[v] for row,v in zip(a,b)];n=len(b)
 for j in range(n):
  k=max(range(j,n),key=lambda k:abs(a[k][j]));a[j],a[k]=a[k],a[j];v=a[j][j]
  if abs(v)<1e-28:raise ValueError('singular')
  a[j]=[x/v for x in a[j]]
  for i in range(n):
   if i!=j:
    v=a[i][j];a[i]=[x-v*y for x,y in zip(a[i],a[j])]
 return [a[i][-1] for i in range(n)]
def deriv(f,x):
 cols=[]
 for i,t in enumerate(x):
  h=1e-6*max(abs(t),1e-2);a=x.copy();b=x.copy();a[i]+=h;b[i]-=h;cols.append(mul(sub(f(a),f(b)),.5/h))
 return list(zip(*cols))
def newton(f,x,least=False):
 for iteration in range(35):
  r=f(x);jac=deriv(f,x)
  if norm(r)<1e-12:break
  if least:
   jt=list(zip(*jac));a=[[dot(c,d) for d in jt] for c in jt];b=[-dot(c,r) for c in jt]
  else:a=jac;b=mul(r,-1)
  try:delta=solve(a,b)
  except ValueError:break
  accepted=False
  for power in range(12):
   y=add(x,mul(delta,2.**-power))
   if norm(f(y))<norm(r):x=y;accepted=True;break
  if not accepted or norm(delta)<1e-13:break
 return x,norm(f(x))
def spline(k,p,c,t):
 j=min(len(c)-1,max(p,bisect.bisect_right(k,t)-1));w=[list(v) for v in c[j-p:j+1]]
 for r in range(1,p+1):
  for i in range(p,r-1,-1):
   a,b=k[i+j-p],k[i+1+j-r];q=(t-a)/(b-a) if b!=a else 0.;w[i]=add(mul(w[i-1],1-q),mul(w[i],q))
 return w[p]
class Model:
 def __init__(self,mode,prefix):
  text=Z.read(f'chamfer/{mode}/C1/capture/{prefix}.input.step').decode();self.d={i:v.strip() for i,v in re.findall(r'#(\d+)\s*=\s*(.*?);',text,re.S)}
 @lru_cache(None)
 def entity(self,i):
  v=self.d[i];args=ast.literal_eval(re.sub(r'\.[A-Z_0-9]+\.',lambda m:repr(m[0]),re.sub(r'#(\d+)',lambda m:repr(m[1]),v[v.index('('):])))
  return v.split('(')[0],args
 def point(self,i):return list(self.entity(i)[1][1])
 def vector(self,i):
  _,v=self.entity(i);return mul(self.point(v[1]),v[2])
 @lru_cache(None)
 def curve(self,i):
  typ,v=self.entity(i)
  if typ=='B_SPLINE_CURVE_WITH_KNOTS':
   k=[t for t,n in zip(v[7],v[6]) for _ in range(n)];c=[self.point(j) for j in v[2]];return lambda t:spline(k,v[1],c,t),(k[0],k[-1])
  if typ=='LINE':
   p=self.point(v[1]);d=self.vector(v[2]);return lambda t:add(p,mul(d,t)),(-1.,1.)
  raise ValueError(typ)
 @lru_cache(None)
 def surface(self,i):
  typ,v=self.entity(i)
  if typ=='SURFACE_OF_LINEAR_EXTRUSION':
   c,dom=self.curve(v[1]);d=self.vector(v[2]);return lambda x:add(c(x[0]),mul(d,x[1])),[dom,(-.2,.2)]
  if typ=='B_SPLINE_SURFACE_WITH_KNOTS':
   ku=[t for t,n in zip(v[10],v[8]) for _ in range(n)];kv=[t for t,n in zip(v[11],v[9]) for _ in range(n)];c=[[self.point(i) for i in row] for row in v[3]]
   return lambda x:spline(ku,v[1],[spline(kv,v[2],row,x[1]) for row in c],x[0]),[(ku[0],ku[-1]),(kv[0],kv[-1])]
  raise ValueError(typ)
 def nearest(self,sid,p,seed=None):
  f,dom=self.surface(sid)
  if seed is None:seed=min(([a+(b-a)*i/8,c+(d-c)*j/8] for (a,b),(c,d) in [dom] for i in range(9) for j in range(9)),key=lambda x:norm(sub(f(x),p)))
  return newton(lambda x:sub(f(x),p),seed,True)
 def vertex(self,vid):return self.point(self.entity(vid)[1][1])
 def three(self,vertex,sids):
  p=self.vertex(vertex);surfaces=[self.surface(s)[0] for s in sids];seed=sum([self.nearest(s,p)[0] for s in sids],[])
  def f(x):
   pts=[surface(x[2*i:2*i+2]) for i,surface in enumerate(surfaces)];return sub(pts[0],pts[1])+sub(pts[0],pts[2])
  uv,residual=newton(f,seed);q=surfaces[0](uv[:2]);return dict(vertex=vertex,supports=sids,original=p,reconstructed=q,shift=norm(sub(p,q)),residual=residual,uv=uv,domains=[self.surface(s)[1] for s in sids])
 def two(self,curveid,sids,t):
  curve,dom=self.curve(curveid);p=curve(t);h=1e-6*(dom[1]-dom[0]);tan=mul(sub(curve(t+h),curve(t-h)),.5/h);tan=mul(tan,1/norm(tan));surfaces=[self.surface(s)[0] for s in sids];seed=sum([self.nearest(s,p)[0] for s in sids],[])
  def f(x):
   a,b=surfaces[0](x[:2]),surfaces[1](x[2:]);return sub(a,b)+[dot(sub(mul(add(a,b),.5),p),tan)]
  uv,residual=newton(f,seed);q=surfaces[0](uv[:2]);return dict(curve=curveid,supports=sids,t=t,original=p,reconstructed=q,shift=norm(sub(p,q)),residual=residual,uv=uv)
if __name__=='__main__':
 out={};m=Model('dist_angle_sequence','002');out['angle002_vertices']=[m.three('24',['32','69','354']),m.three('89',['32','694','354'])];print(json.dumps(out,indent=2));Path('target/d-c1-reconstruction-probe-2026-10-24.json').write_text(json.dumps(out,indent=2))
