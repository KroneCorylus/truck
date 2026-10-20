from pathlib import Path
import json,subprocess,os,math
root=Path.cwd();env=dict(os.environ,LD_LIBRARY_PATH=str(root/'target/occt-sdk/usr/lib64'));base=root/'target/procedural-cylinders-controls';native={q['mode']:q for q in json.load(open('target/native-procedural-cylinders/summary.json'))};records=json.loads((base/'summary.json').read_text())
for q in records:
 ref=native[q['mode']];assert ref['native_input_unchanged'] and q['input_unchanged'] and q['procedural_support_unchanged'] and q['geometry_consistent']
 for label in ['input','result']:
  output=base/f"{q['mode']}.{label}.comparison.json"
  with output.with_suffix('.log').open('w') as log:subprocess.run(['target/occt-cone-reference-plugin/compare_shapes',ref['prefix']+f'.{label}.brep',str(base/f"{q['mode']}.{label}.step"),str(output)],env=env,stdout=log,stderr=subprocess.STDOUT,check=True,timeout=120)
  m=json.loads(output.read_text());assert m['native_valid'] and m['truck_step_valid'] and m['native_solids']==m['truck_solids']==1,(q,label,m)
  assert max(m['native_to_truck_max_distance'],m['truck_to_native_max_distance'])<.001
  q[label+'_comparison']=m
 s,r=q['size'],q['radius'];input_expected=s**3-r*r*(1-math.pi/4)*s;input_error=abs(q['input_comparison']['truck_volume']-input_expected);assert input_error<input_expected*1e-6 and input_error<(math.pi/2*r*s)*5e-5
 assert abs(q['input_comparison']['native_volume']-input_expected)<1e-7 and abs(q['result_comparison']['native_volume']-q['expected_volume'])<1e-7
 q.update(expected_input_volume=input_expected,input_volume_error=input_error)
 area=math.pi/2*r*(3*(s-r)+r);error=abs(q['result_comparison']['truck_volume']-q['expected_volume']);assert error<q['expected_volume']*1e-6 and error<area*5e-5
 q.update(volume_error=error,material_error_ppm=error/q['expected_volume']*1e6,analytic_curved_area=area,curved_area_times_step_tolerance=area*5e-5,native_reference=ref)
 with (base/f"{q['mode']}.surfaces.log").open('w') as log:surfaces=json.loads(subprocess.check_output(['target/cylinder-angle-surface-oracle',str(base/f"{q['mode']}.result.step")],env=env,stderr=log))[0]
 (base/f"{q['mode']}.surfaces.json").write_text(json.dumps(surfaces,indent=2)+'\n')
 spheres=[f for f in surfaces['faces'] if f['type']==3];assert len(spheres)==1;center=(r,r,s-r)
 radius_errors=[abs(math.dist(p,center)-r) for f in spheres for p in f['samples']]
 cylinders=[f for f in surfaces['faces'] if f['type']==1 and all(p[2]>s-r-1e-7 for p in f['vertices']) and any(abs(p[2]-s)<1e-7 for p in f['vertices'])];assert len(cylinders)==2
 for f in cylinders:
  axis=min([0,1],key=lambda a:max(abs(math.hypot(p[a]-r,p[2]-(s-r))-r) for p in f['vertices']))
  radius_errors.extend(abs(math.hypot(p[axis]-r,p[2]-(s-r))-r) for p in f['samples'])
 assert max(radius_errors)<1e-7
 contacts=[(r,r,s),(s,r,s),(r,s,s),(r,0.,s-r),(0.,r,s-r),(s,0.,s-r),(0.,s,s-r)]
 contact_error=max(min(math.dist(p,v) for v in surfaces['vertices']) for p in contacts);assert contact_error<1e-7
 assert max(min(math.dist(p,v) for v in contacts) for f in spheres+cylinders for p in f['vertices'])<1e-7
 old=[f for f in surfaces['faces'] if f['type']==6];assert len(old)==1
 old_error=max(abs(math.hypot(p[0]-r,p[1]-r)-r) for f in old for p in f['samples']);assert old_error<5e-5
 q.update(max_new_radius_error=max(radius_errors),new_surface_samples=len(radius_errors),max_contact_error=contact_error,max_retained_support_radius_error=old_error,retained_support_samples=sum(len(f['samples']) for f in old))
 print(q['mode'],'volume',error,'ppm',q['material_error_ppm'],'distance',max(q['result_comparison'][k] for k in ['native_to_truck_max_distance','truck_to_native_max_distance']),'radius',max(radius_errors),'old_radius',old_error,flush=True)
Path('target/procedural-cylinders-comparison-summary.json').write_text(json.dumps(records,indent=2)+'\n')
