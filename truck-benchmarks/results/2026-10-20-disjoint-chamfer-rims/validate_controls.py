from pathlib import Path
import json,subprocess,os,math,hashlib
root=Path.cwd();raw=root/'target/native-disjoint-rims';proof=root/'truck-benchmarks/results/2026-10-20-disjoint-chamfer-rims'
env=dict(os.environ,LD_LIBRARY_PATH=str(root/'target/occt-sdk/usr/lib64'));sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
v=json.loads((proof/'validation.json').read_text())
for p,h in v['source_sha256'].items():assert sha(p)==h,p
assert sha('target/release/examples/replay_occt_blend')==v['replay_sha256']
records=json.loads((raw/'summary.json').read_text())
for q in records:
 assert q['native_input_unchanged'] and q['native']['native_valid'] and q['native']['solids']==1
 prefix=Path(q['prefix']);inputs={str(p):sha(p) for p in prefix.parent.glob(prefix.name+'.input.*')}
 with Path(str(prefix)+'.truck.log').open('w') as log:subprocess.run([str(root/'target/release/examples/replay_occt_blend'),str(prefix)],stdout=log,stderr=subprocess.STDOUT,check=True,timeout=120)
 result=json.loads(Path(str(prefix)+'.truck.json').read_text())['result'];assert result['status']=='operation-succeeded',result
 assert result['selected_edges']>=2 and all(result[k] for k in ['input_unchanged','geometry_consistent','mesh_closed','step_prepared'])
 output=Path(str(prefix)+'.comparison.json')
 with output.with_suffix('.log').open('w') as log:subprocess.run([str(root/'target/occt-cone-reference-plugin/compare_shapes'),str(prefix)+'.result.brep',str(prefix)+'.truck.step',str(output)],env=env,stdout=log,stderr=subprocess.STDOUT,check=True,timeout=120)
 comparison=json.loads(output.read_text());assert comparison['native_valid'] and comparison['truck_step_valid'] and comparison['native_solids']==comparison['truck_solids']==1
 assert abs(comparison['truck_volume']-q['expected_volume'])<1e-5,(q,comparison)
 assert max(comparison['native_to_truck_max_distance'],comparison['truck_to_native_max_distance'])<.001,comparison
 surfaces=json.loads(subprocess.check_output([str(root/'target/cylinder-angle-surface-oracle'),str(prefix)+'.truck.step'],env=env,stderr=subprocess.DEVNULL))[0]
 Path(str(prefix)+'.truck-surfaces.json').write_text(json.dumps(surfaces,indent=2)+'\n')
 cone_checks=[]
 for end,(radial,axial) in enumerate(q['distances']):
  cap=q['height'] if end==0 else 0.;side=cap-axial if end==0 else axial;radius=q['radius']
  def on_contact(p,cylinder):return abs(math.hypot(p[0],p[1])-(radius if cylinder else radius-radial))<1e-6 and abs(p[2]-(side if cylinder else cap))<1e-6
  bevels=[f for f in surfaces['faces'] if all(on_contact(p,True) or on_contact(p,False) for p in f['vertices']) and all(any(on_contact(p,c) for p in f['vertices']) for c in [False,True])]
  assert bevels,(q,end)
  errors=[]
  for face in bevels:
   for p in face['samples']:
    t=(cap-p[2]) if end==0 else p[2]
    errors.append(abs(math.hypot(p[0],p[1])-(radius-radial+radial*t/axial)))
  assert max(errors)<1e-5,(q,end,max(errors))
  cone_checks.append(dict(end=end,faces=len(bevels),sample_count=len(errors),max_cone_error=max(errors)))
 for p,h in inputs.items():assert sha(p)==h,p
 q.update(truck=result,comparison=comparison,cone_checks=cone_checks,input_sha256=inputs)
 print(q['shape'],q['mode'],'Truck volume error',comparison['truck_volume']-q['expected_volume'],'sampled distance',max(comparison['native_to_truck_max_distance'],comparison['truck_to_native_max_distance']),'cone error',max(c['max_cone_error'] for c in cone_checks),flush=True)
for p,h in v['source_sha256'].items():assert sha(p)==h,p
Path('target/disjoint-rims-validated-summary.json').write_text(json.dumps(records,indent=2)+'\n')
