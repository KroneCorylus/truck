from pathlib import Path
import json,math,subprocess,os
dot=lambda a,b:sum(x*y for x,y in zip(a,b))
sub=lambda a,b:[x-y for x,y in zip(a,b)]
root=Path('target/occt-capture-longitudinal-cylinder-angles');rows=[]
for case in ['chamfer/dist_angle_sequence/A6','chamfer/dist_angle_sequence/A8']:
 for serial in ['005','006']:
  prefix=root/case/'capture'/serial;t=json.loads(Path(str(prefix)+'.input.json').read_text());assert len(t['selection'])==1
  task=t['selection'][0];c=task['reference_cylinder'];o=c['origin'];r=c['radius'];axis=c['axis']
  seed=min(task['points'],key=lambda p:dot(p,axis))
  measured=json.loads(subprocess.check_output(['target/solid-contact-oracle',str(prefix)+'.result.brep',str(prefix)+'.truck.step'],env=dict(os.environ,LD_LIBRARY_PATH='target/occt-sdk/usr/lib64'),stderr=subprocess.DEVNULL))
  row=dict(case=case,capture=serial,requested_angle=task['angle_degrees'],requested_chord=task['distance'])
  for kind,m in zip(['native','truck'],measured):
   contacts=[p for p in m['vertices'] if abs(dot(sub(p,seed),axis))<1e-8 and abs(math.sqrt(dot(sub(p,o),sub(p,o))-dot(sub(p,o),axis)**2)-r)<1e-7 and abs(math.dist(p,seed)-task['distance'])<1e-7]
   unique=[]
   for p in contacts:
    if not any(math.dist(p,q)<1e-8 for q in unique):unique.append(p)
   assert len(unique)==1,(row,kind,unique)
   q=unique[0];planes=[p for p in m['bevel_planes'] if abs(dot(p['normal'],axis))<1e-8 and abs(dot(sub(q,p['origin']),p['normal']))<1e-7]
   assert len(planes)==1,(row,kind,planes)
   n=planes[0]['normal'];angle=math.degrees(math.acos(min(1,abs(dot(n,sub(q,o))/r))))
   row[kind]=dict(contact=q,plane=planes[0],contact_angle=angle,chord=math.dist(q,seed))
   if kind=='truck':assert abs(angle-task['angle_degrees'])<1e-7,(row,angle)
  rows.append(row);print(case,serial,'requested',row['requested_angle'],'native',row['native']['contact_angle'],'truck',row['truck']['contact_angle'])
Path('audits/fillet-chamfer/capture/2026-10-20-longitudinal-cylinder-angles/contact-angle-checks.json').write_text(json.dumps(rows,indent=2)+'\n')
