from pathlib import Path
import json,subprocess,os,hashlib
root=Path.cwd();raw=root/'target/native-reordered-loops';proof=root/'truck-benchmarks/results/2026-10-20-planar-loop-order'
env=dict(os.environ,LD_LIBRARY_PATH=str(root/'target/occt-sdk/usr/lib64'));sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
v=json.loads((proof/'validation.json').read_text())
for p,h in v['source_sha256'].items():assert sha(p)==h,p
assert sha('target/release/examples/replay_occt_blend')==v['replay_sha256']
records=json.loads((raw/'summary.json').read_text())
for q in records:
 assert q['native_input_unchanged']==(q['shape']=='outer') and q['native']['native_valid'] and q['native']['solids']==1
 prefix=Path(q['prefix']);inputs={str(p):sha(p) for p in prefix.parent.glob(prefix.name+'.input.*')}
 with Path(str(prefix)+'.truck.log').open('w') as log:subprocess.run([str(root/'target/release/examples/replay_occt_blend'),str(prefix)],stdout=log,stderr=subprocess.STDOUT,check=True,timeout=120)
 result=json.loads(Path(str(prefix)+'.truck.json').read_text())['result'];assert result['status']=='operation-succeeded',result
 assert result['selected_edges']==1 and all(result[k] for k in ['input_unchanged','geometry_consistent','mesh_closed','step_prepared'])
 output=Path(str(prefix)+'.comparison.json')
 with output.with_suffix('.log').open('w') as log:subprocess.run([str(root/'target/occt-cone-reference-plugin/compare_shapes'),str(prefix)+'.result.brep',str(prefix)+'.truck.step',str(output)],env=env,stdout=log,stderr=subprocess.STDOUT,check=True,timeout=120)
 comparison=json.loads(output.read_text());assert comparison['native_valid'] and comparison['truck_step_valid'] and comparison['native_solids']==comparison['truck_solids']==1
 assert abs(comparison['truck_volume']-q['expected_volume'])<1e-6,(q,comparison)
 assert max(comparison['native_to_truck_max_distance'],comparison['truck_to_native_max_distance'])<.001,comparison
 surfaces=json.loads(subprocess.check_output([str(root/'target/cylinder-angle-surface-oracle'),str(prefix)+'.truck.step'],env=env,stderr=subprocess.DEVNULL))[0]
 Path(str(prefix)+'.truck-surfaces.json').write_text(json.dumps(surfaces,indent=2)+'\n')
 wall,cap=(2.,2.-q['planar']) if q['shape']=='inner' else (0.,q['planar']);axial=q['axial'];ends=(2.,4.) if q['shape']=='inner' else (0.,10.)
 def contact(p,side):return abs(p[1]-(wall if side else cap))<1e-7 and abs(p[2]-(5-axial if side else 5))<1e-7
 bevels=[f for f in surfaces['faces'] if len(f['vertices'])==4 and all(contact(p,True) or contact(p,False) for p in f['vertices']) and all(any(contact(p,c) for p in f['vertices']) for c in [False,True])]
 assert len(bevels)==1,q
 assert all(min(abs(p[0]-x) for x in ends)<1e-7 for p in bevels[0]['vertices'])
 errors=[abs(p[1]-(cap+(wall-cap)*(5-p[2])/axial)) for p in bevels[0]['samples']];assert max(errors)<1e-7,(q,max(errors))
 for p,h in inputs.items():assert sha(p)==h,p
 q.update(truck=result,comparison=comparison,contact_check=dict(vertices=4,sample_count=len(errors),max_plane_error=max(errors)),input_sha256=inputs)
 print(q['shape'],q['mode'],'Truck volume error',comparison['truck_volume']-q['expected_volume'],'sampled distance',max(comparison['native_to_truck_max_distance'],comparison['truck_to_native_max_distance']),'plane error',max(errors),flush=True)
for p,h in v['source_sha256'].items():assert sha(p)==h,p
Path('target/planar-loop-order-validated-summary.json').write_text(json.dumps(records,indent=2)+'\n')
