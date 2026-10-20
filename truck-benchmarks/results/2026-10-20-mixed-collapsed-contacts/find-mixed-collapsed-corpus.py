from pathlib import Path
import json,math
root=Path('target/occt-capture-collapsed-fillet-contacts');m=json.loads((root/'results.json').read_text());out=[]
sub=lambda a,b:[x-y for x,y in zip(a,b)]
cross=lambda a,b:[a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]]
norm=lambda a:math.sqrt(sum(x*x for x in a))
for case in m['cases']:
 for q in case['captures']:
  if q.get('truck',{}).get('result',{}).get('status')!='operation-failed':continue
  p=root/case['id']/'capture'/f"{q['prefix']}.input.json";task=json.loads(p.read_text())
  if task['operation']!='fillet':continue
  for c in task.get('contours',[]):
   r=c.get('constant_radius');edges=c['edges'];radii=[]
   if not r or any(e['curve_type'] not in [0,1] for e in edges):continue
   for e in edges:
    pts=e['points']
    if e['curve_type']!=1 or len(pts)<3:continue
    a,b,d=pts[0],pts[len(pts)//3],pts[2*len(pts)//3];ab=sub(a,b);ad=sub(a,d);bd=sub(b,d);denom=2*norm(cross(ab,ad))
    if denom:radii.append(norm(ab)*norm(ad)*norm(bd)/denom)
   if radii and min(abs(x-r) for x in radii)<r*1e-7 and max(radii)>r*(1+1e-6):out.append(dict(case=case['id'],capture=q['prefix'],contour=c['index'],radius=r,circle_radii=radii,edges=len(edges)))
Path('target/mixed-collapsed-corpus-candidates.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps(out,indent=2))
