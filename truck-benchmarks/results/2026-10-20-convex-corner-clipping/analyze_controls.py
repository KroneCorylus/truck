from pathlib import Path
import itertools,json,math
root=Path('target/native-convex-corner-clip');records=json.loads((root/'summary.json').read_text())
dot=lambda a,b:sum(x*y for x,y in zip(a,b))
cross=lambda a,b:[a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]]
for q in records:
 ds=q['distances'];secondary=ds if q['angles'] is None else [d*math.tan(math.radians(a)) for d,a in zip(ds,q['angles'])]
 planes=[([-1.,0.,0.],0.),([1.,0.,0.],1.),([0.,-1.,0.],0.),([0.,1.,0.],1.),([0.,0.,-1.],0.),([0.,0.,1.],1.),([-1/secondary[0],0.,1/ds[0]],1/ds[0]-1),([-1/secondary[1],1/ds[1],0.],1/ds[1]-1),([0.,1/secondary[2],1/ds[2]],1/secondary[2]+1/ds[2]-1)]
 planes=[([x/math.sqrt(dot(n,n)) for x in n],d/math.sqrt(dot(n,n))) for n,d in planes];vertices=[]
 for (a,da),(b,db),(c,dc) in itertools.combinations(planes,3):
  bc,ca,ab=cross(b,c),cross(c,a),cross(a,b);det=dot(a,bc)
  if abs(det)<1e-12:continue
  p=[(da*bc[j]+db*ca[j]+dc*ab[j])/det for j in range(3)]
  if all(dot(n,p)<=d+1e-10 for n,d in planes) and all(math.dist(p,v)>1e-9 for v in vertices):vertices.append(p)
 volume=0.;faces=[]
 for n,d in planes:
  points=[p for p in vertices if abs(dot(n,p)-d)<1e-9]
  if len(points)<3:continue
  center=[sum(p[j] for p in points)/len(points) for j in range(3)];u=[points[0][j]-center[j] for j in range(3)];v=cross(n,u)
  points.sort(key=lambda p:math.atan2(dot([p[j]-center[j] for j in range(3)],v),dot([p[j]-center[j] for j in range(3)],u)))
  area=abs(sum(dot(n,cross(a,b)) for a,b in zip(points,points[1:]+points[:1])))/2;volume+=area*d/3;faces.append(dict(normal=n,offset=d,vertices=points,area=area))
 error=q['native']['volume']-volume;q.update(expected_volume=volume,halfspace_vertices=vertices,halfspace_faces=faces,native_volume_error=error)
 print(Path(q['prefix']).name,'volume',volume,'native error',error,'input unchanged',q['native_input_unchanged'],'faces',len(faces),'vertices',len(vertices))
(root/'analysis.json').write_text(json.dumps(records,indent=2)+'\n')
