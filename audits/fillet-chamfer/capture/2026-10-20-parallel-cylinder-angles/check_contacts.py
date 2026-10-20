from pathlib import Path
import json,math,subprocess,os,hashlib
prefix=Path('target/occt-capture-parallel-cylinder-angles/chamfer/dist_angle/A2/capture/001');dest=Path('audits/fillet-chamfer/capture/2026-10-20-parallel-cylinder-angles');task=json.loads(Path(str(prefix)+'.input.json').read_text())
data=json.loads(subprocess.check_output(['target/cylinder-angle-surface-oracle',str(prefix)+'.result.brep',str(prefix)+'.truck.step'],env=dict(os.environ,LD_LIBRARY_PATH='target/occt-sdk/usr/lib64'),stderr=subprocess.DEVNULL));records=[]
for i,s in enumerate(task['selection']):
 seed=s['points'][0];record=dict(selection=i,seed=seed,requested_distance=s['distance'],requested_angle=s['angle_degrees'])
 for kind,m in zip(['native','truck'],data):
  contacts=[v for v in m['vertices'] if abs(v[2])<1e-8 and abs(math.dist(v,seed)-s['distance'])<1e-7]
  ref=s.get('reference_cylinder') or s['reference_plane'];cylinder=bool(s.get('reference_cylinder'))
  if cylinder:contacts=[v for v in contacts if abs(math.hypot(v[0]-ref['origin'][0],v[1]-ref['origin'][1])-ref['radius'])<1e-7]
  else:contacts=[v for v in contacts if abs(sum((v[j]-ref['origin'][j])*ref['normal'][j] for j in range(3)))<1e-7]
  assert len(contacts)==1,(i,kind,contacts)
  contact=contacts[0];refnormal=[(contact[j]-ref['origin'][j])/ref['radius'] for j in range(3)] if cylinder else ref['normal']
  candidates=[]
  for face in m['faces']:
   if not any(math.dist(contact,p)<1e-7 for p in face['vertices']):continue
   points=[p for p in face['vertices'] if abs(p[2])<1e-8]
   if len(points)!=2:continue
   delta=[points[1][j]-points[0][j] for j in range(2)];length=math.hypot(*delta);normal=[delta[1]/length,-delta[0]/length,0.]
   residual=max(abs(sum((p[j]-points[0][j])*normal[j] for j in range(3))) for p in face['samples'])
   if residual>1e-7:continue
   angle=math.degrees(math.acos(min(1.,abs(sum(a*b for a,b in zip(normal,refnormal))))))
   if angle<1e-5:continue
   candidates.append(dict(contact=contact,distance=math.dist(contact,seed),angle_degrees=angle,plane_residual=residual,surface_type=face['type'],face_vertices=face['vertices']))
  assert len(candidates)==1,(i,kind,candidates)
  record[kind]=candidates[0]
  if kind=='truck':assert abs(record[kind]['angle_degrees']-s['angle_degrees'])<1e-7,(i,record)
 records.append(record);print(i,'native',record['native']['angle_degrees'],'truck',record['truck']['angle_degrees'])
(dest/'contact-angle-checks.json').write_text(json.dumps(records,indent=2)+'\n')
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
(dest/'contact-checks-metadata.json').write_text(json.dumps(dict(inputs={str(prefix)+ext:sha(str(prefix)+ext) for ext in ['.input.json','.result.brep','.truck.step']},oracle_source_sha256=sha('target/cylinder-angle-surface-oracle.cpp'),oracle_executable_sha256=sha('target/cylinder-angle-surface-oracle')),indent=2)+'\n')
