from pathlib import Path
import json,subprocess,os,math
root=Path.cwd();env=dict(os.environ,LD_LIBRARY_PATH=str(root/'target/occt-sdk/usr/lib64'));base=root/'target/mixed-collapsed-controls';native={q['mode']:q for q in json.load(open('target/native-mixed-collapsed-rims/summary.json'))};records=json.loads((base/'summary.json').read_text())
for q in records:
 ref=native[q['mode']];assert ref['native_input_unchanged'] and q['input_unchanged'] and q['geometry_consistent']
 for label in ['input','result']:
  output=base/f"{q['mode']}.{label}.comparison.json"
  with output.with_suffix('.log').open('w') as log:subprocess.run(['target/occt-cone-reference-plugin/compare_shapes',ref['prefix']+f'.{label}.brep',str(base/f"{q['mode']}.{label}.step"),str(output)],env=env,stdout=log,stderr=subprocess.STDOUT,check=True,timeout=120)
  m=json.loads(output.read_text());assert m['native_valid'] and m['truck_step_valid'] and m['native_solids']==m['truck_solids']==1;assert max(m['native_to_truck_max_distance'],m['truck_to_native_max_distance'])<.001;q[label+'_comparison']=m
 initial=960.-(1-math.pi/4)*8*18;assert abs(q['input_comparison']['native_volume']-initial)<1e-7 and abs(q['input_comparison']['truck_volume']-initial)<1e-7
 assert abs(q['result_comparison']['native_volume']-q['expected_volume'])<1e-7
 error=abs(q['result_comparison']['truck_volume']-q['expected_volume']);assert error<(1e-7 if q['radius']==1. else q['expected_volume']*1e-6)
 q.update(volume_error=error,native_reference=ref)
 if q['radius']==1.:
  with (base/f"{q['mode']}.surfaces.log").open('w') as log:surfaces=json.loads(subprocess.check_output(['target/cylinder-angle-surface-oracle',str(base/f"{q['mode']}.result.step")],env=env,stderr=log))[0]
  (base/f"{q['mode']}.surfaces.json").write_text(json.dumps(surfaces,indent=2)+'\n')
  corners=[f for f in surfaces['faces'] if f['type'] in [3,4]];assert [f['type'] for f in corners].count(3)==1 and [f['type'] for f in corners].count(4)==3
  centers=[(1.,1.,7.,0.),(10.,2.,7.,1.),(9.,7.,7.,2.),(2.,8.,7.,1.)]
  def residual(c,p):return abs(math.hypot(math.hypot(p[0]-c[0],p[1]-c[1])-c[3],p[2]-c[2])-1.)
  errors=[]
  for f in corners:
   c=min(centers,key=lambda c:max(residual(c,p) for p in f['samples']));errors.extend(residual(c,p) for p in f['samples'])
  strips=[f for f in surfaces['faces'] if f['type']==1 and all(p[2]>7.-1e-7 for p in f['vertices']) and any(abs(p[2]-8.)<1e-7 for p in f['vertices'])];assert len(strips)==4
  for f in strips:
   axis,offset=min([(0,1.),(0,11.),(1,1.),(1,9.)],key=lambda c:max(abs(math.hypot(p[c[0]]-c[1],p[2]-7.)-1.) for p in f['vertices']))
   errors.extend(abs(math.hypot(p[axis]-offset,p[2]-7.)-1.) for p in f['samples'])
  assert len(errors)==648 and max(errors)<1e-7
  rim=[(0.,1.,1.,1.,1.),(1.,0.,1.,1.,1.),(10.,0.,10.,2.,2.),(12.,2.,10.,2.,2.),(12.,7.,9.,7.,3.),(9.,10.,9.,7.,3.),(2.,10.,2.,8.,2.),(0.,8.,2.,8.,2.)]
  contacts=[p for x,y,cx,cy,r in rim for p in [(x,y,7.),(x-(x-cx)/r,y-(y-cy)/r,8.)]]
  contact_error=max(min(math.dist(p,v) for v in surfaces['vertices']) for p in contacts);assert contact_error<1e-7
  assert max(min(math.dist(p,v) for v in contacts) for f in corners+strips for p in f['vertices'])<1e-7
  q.update(surface_samples=len(errors),max_radius_error=max(errors),max_contact_error=contact_error)
 print(q['mode'],'volume',error,'distance',max(q['result_comparison'][k] for k in ['native_to_truck_max_distance','truck_to_native_max_distance']),flush=True)
Path('target/mixed-collapsed-comparison-summary.json').write_text(json.dumps(records,indent=2)+'\n')
