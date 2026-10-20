from pathlib import Path
import json,subprocess,os,math
out=Path('audits/fillet-chamfer/capture/2026-10-20-disjoint-chamfer-rims'); rows=[]
for case in ['B3','B4','C2','C3','C4']:
 p=Path('target/occt-capture-disjoint-chamfer-rims/chamfer/dist_dist')/case/'capture/001'
 task=json.loads(Path(str(p)+'.input.json').read_text())
 for i,s in enumerate(task['selection']):
  pts=s['points']; a,b=s['distances']; plane,cone=(b,a) if s.get('reference_cone') else (a,b)
  for kind,ext in [('native','.result.brep'),('truck','.truck.step')]:
   rows.append(dict(case=case,selection=i,kind=kind,input=str(p)+'.input.brep',result=str(p)+ext,points=[pts[0],pts[len(pts)//2],pts[-1]],limit=2*max(a,b),expected_plane=plane,expected_cone=cone,reference='cone' if s.get('reference_cone') else 'plane',first=a,second=b))
tasks=out/'cone-contact-tasks.txt'; tasks.write_text(''.join(f"{i} {r['input']} {r['result']} "+' '.join(str(v) for p in r['points'] for v in p)+f" {r['limit']}\n" for i,r in enumerate(rows)))
with (out/'cone-contact-log.txt').open('w') as log:subprocess.run(['target/cone-contact-oracle',str(tasks),str(out/'cone-contact-measurements.json')],env=dict(os.environ,LD_LIBRARY_PATH=str(Path('target/occt-sdk/usr/lib64').resolve())),stdout=log,stderr=subprocess.STDOUT,check=True)
ms=json.loads((out/'cone-contact-measurements.json').read_text())
for r,m in zip(rows,ms):
 assert m['valid'] and len(m['planar_setbacks'])==len(m['cone_setbacks'])==1,(r,m)
 r['measurement']=m
 r['max_setback_error']=max(abs(m['planar_setbacks'][0]-r['expected_plane'])+m['planar_spread']/2,abs(m['cone_setbacks'][0]-r['expected_cone'])+m['cone_spread']/2)
 if r['kind']=='truck':assert r['max_setback_error']<1e-6,(r,m)
 elif r['max_setback_error']>1e-6:
  observed=m['cone_setbacks' if r['reference']=='cone' else 'planar_setbacks'][0]; folded=r['first']*r['second']/(2*r['first']*math.cos(math.pi/4)-r['second']);assert abs(observed-folded)<1e-10,(r,m,folded)
  r['native_first_setback_fold']=folded
 print(r['case'],r['selection'],r['kind'],m['planar_setbacks'],m['cone_setbacks'],r['max_setback_error'])
(out/'cone-contact-analysis.json').write_text(json.dumps(rows,indent=2)+'\n')
