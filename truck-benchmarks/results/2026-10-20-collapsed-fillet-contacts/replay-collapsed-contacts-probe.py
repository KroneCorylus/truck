from pathlib import Path
import json,subprocess,os,math
root=Path.cwd();env=dict(os.environ,LD_LIBRARY_PATH=str(root/'target/occt-sdk/usr/lib64'));records=[]
for folder,closed in [('native-collapsed-contacts',False),('native-collapsed-contacts-closed',True)]:
 for q in json.load(open(root/'target'/folder/'summary.json')):
  prefix=Path(q['prefix']);q['closed']=closed
  with Path(str(prefix)+'.truck.log').open('w') as log:subprocess.run(['target/release/examples/replay_occt_blend',str(prefix)],stdout=log,stderr=subprocess.STDOUT,check=True,timeout=120)
  result=json.loads(Path(str(prefix)+'.truck.json').read_text())['result'];assert result['status']=='operation-succeeded',(q,result)
  assert all(result[k] for k in ['input_unchanged','geometry_consistent','mesh_closed','step_prepared'])
  assert result['selected_edges']==(8 if closed else 3) and result['step_fitting_tolerance']==5e-5
  output=Path(str(prefix)+'.comparison.json')
  with Path(str(prefix)+'.comparison.log').open('w') as log:subprocess.run(['target/occt-cone-reference-plugin/compare_shapes',str(prefix)+'.result.brep',str(prefix)+'.truck.step',str(output)],env=env,stdout=log,stderr=subprocess.STDOUT,check=True,timeout=120)
  comparison=json.loads(output.read_text());assert comparison['native_valid'] and comparison['truck_step_valid'] and comparison['native_solids']==comparison['truck_solids']==1
  distance=max(comparison['native_to_truck_max_distance'],comparison['truck_to_native_max_distance']);assert distance<.001
  error=abs(comparison['truck_volume']-q['expected_volume']);assert error<1e-7
  with Path(str(prefix)+'.surfaces.log').open('w') as log:
   surfaces=json.loads(subprocess.check_output(['target/cylinder-angle-surface-oracle',str(prefix)+'.truck.step'],env=env,stderr=log))[0]
  Path(str(prefix)+'.surfaces.json').write_text(json.dumps(surfaces,indent=2)+'\n')
  size,r=q['size'],q['radius'];centers=[(r,r,size-r)] if not closed else [(x,y,size-r) for x in [r,size-r] for y in [r,size-r]]
  spheres=[f for f in surfaces['faces'] if f['type']==3];assert len(spheres)==len(centers)
  radius_errors=[];used=set()
  for f in spheres:
   nearest=min(range(len(centers)),key=lambda i:max(abs(math.dist(p,centers[i])-r) for p in f['vertices']));assert nearest not in used;used.add(nearest)
   assert max(abs(math.dist(p,centers[nearest])-r) for p in f['vertices'])<1e-7
   radius_errors.extend(abs(math.dist(p,centers[nearest])-r) for p in f['samples'])
  cylinders=[f for f in surfaces['faces'] if f['type']==1 and all(p[2]>size-r-1e-7 for p in f['vertices']) and any(abs(p[2]-size)<1e-7 for p in f['vertices'])]
  assert len(cylinders)==(4 if closed else 2)
  axes=[(0,r),(1,r)]+([(0,size-r),(1,size-r)] if closed else [])
  for f in cylinders:
   component,value=min(axes,key=lambda a:max(abs(math.hypot(p[a[0]]-a[1],p[2]-(size-r))-r) for p in f['vertices']))
   radius_errors.extend(abs(math.hypot(p[component]-value,p[2]-(size-r))-r) for p in f['samples'])
  expected_contacts=[]
  if closed:
   for x,y,z in centers:
    expected_contacts.extend([(x,y,size),(0. if x==r else size,y,size-r),(x,0. if y==r else size,size-r)])
  else:expected_contacts=[(r,r,size),(size,r,size),(r,size,size),(r,0.,size-r),(0.,r,size-r),(size,0.,size-r),(0.,size,size-r)]
  contact_error=max(min(math.dist(p,v) for v in surfaces['vertices']) for p in expected_contacts)
  assert contact_error<1e-7
  assert max(min(math.dist(p,v) for v in expected_contacts) for f in spheres+cylinders for p in f['vertices'])<1e-7
  q['max_contact_error']=contact_error
  assert max(radius_errors)<1e-7
  q.update(truck=result,comparison=comparison,volume_error=error,max_radius_error=max(radius_errors),surface_samples=len(radius_errors))
  records.append(q);print(q['mode'],'closed',closed,'volume',error,'distance',distance,'radius',max(radius_errors),flush=True)
Path('target/collapsed-contacts-replay-summary.json').write_text(json.dumps(records,indent=2)+'\n')
