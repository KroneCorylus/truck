import json,math,subprocess,hashlib,itertools
from pathlib import Path
import sys
out=Path(sys.argv[1]); reader=Path(sys.argv[2])
def add(a,b):return [x+y for x,y in zip(a,b)]
def sub(a,b):return [x-y for x,y in zip(a,b)]
def mul(a,t):return [x*t for x in a]
def cross(a,b):return [a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]]
def unit(a):return mul(a,1/math.sqrt(sum(x*x for x in a)))
def patch(u,v,swap=False):
 # Derived from X+y=d, X+z=d, y+z=h and the pivot section X=d.
 d=.2;h=.1;A=[0,d,d];B=[d,0,d];C=[d,0,h];D=[d,h,0]
 p=[(1-u)*(1-v)*A[i]+u*(1-v)*B[i]+u*v*C[i]+(1-u)*v*D[i] for i in range(3)]
 du=add(mul(sub(B,A),1-v),mul(sub(C,D),v));dv=add(mul(sub(D,A),1-u),mul(sub(C,B),u));n=unit(cross(du,dv))
 if sum(n)<0:n=mul(n,-1)
 if swap:p=[p[0],p[2],p[1]];n=[n[0],n[2],n[1]]
 return p,n
probes=[]
for end in [0,1]:
 for sw in [False,True]:
  for u,v in itertools.product([.2,.5,.8],repeat=2):
   p,n=patch(u,v,sw)
   for e in [-.001,0,.001]:
    q=add(p,mul(n,e));q[0]=1-q[0] if end else q[0]
    probes.append(dict(kind='patch',cap=end,swapped=sw,uv=[u,v],offset=e,point=q,expected=1 if e<0 else 2 if e==0 else 0))
 # Four immediate neighbors plus cap, sampled away from every shared boundary.
 for name,p,n in [('cap-bevel-y',[.1,.1,.22],[1,1,0]),('support-y',[.22,0,.15],[0,1,0]),('axis-bevel',[.22,.05,.05],[0,1,1]),('cap-bevel-z',[.1,.17,.1],[1,0,1]),('cap',[0,.3,.3],[1,0,0])]:
  for e in [-.001,0,.001]:
   q=add(p,mul(unit(n),e));q[0]=1-q[0] if end else q[0]
   probes.append(dict(kind=name,cap=end,offset=e,point=q,expected=1 if e<0 else 2 if e==0 else 0))
results={}
for case in ['A6','A8','A9']:
 for suffix in ['.result.brep','.truck.step']:
  file=out/'annotated'/case/('001'+suffix)
  text=subprocess.check_output([str(reader),str(file),'batch'],input='\n'.join(' '.join(map(str,p['point'])) for p in probes)+'\n',text=True)
  states=[int(l) for l in text.splitlines() if l in ['0','1','2','3']]
  assert len(states)==len(probes)
  results[case+suffix]={'sha256':hashlib.sha256(file.read_bytes()).hexdigest(),'states':states}
  assert all(st==p['expected'] for st,p in zip(states,probes) if not p.get('swapped',False))
 for suffix in ['.truck.step']:
  assert results[case+suffix]['states']==results[case+'.result.brep']['states']
(out/'material.json').write_text(json.dumps({'probes':probes,'results':results},indent=2))
print('All 138 classifier states per target agree; all 84 intended patch/neighbor/cap states pass.')
