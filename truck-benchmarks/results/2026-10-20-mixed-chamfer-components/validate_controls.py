from pathlib import Path
import json,subprocess,os,hashlib
root=Path.cwd();raw=root/'target/native-mixed-disjoint';proof=root/'truck-benchmarks/results/2026-10-20-mixed-chamfer-components'
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
 assert result['selected_edges']==2 and all(result[k] for k in ['input_unchanged','geometry_consistent','mesh_closed','step_prepared'])
 output=Path(str(prefix)+'.comparison.json')
 with output.with_suffix('.log').open('w') as log:subprocess.run([str(root/'target/occt-cone-reference-plugin/compare_shapes'),str(prefix)+'.result.brep',str(prefix)+'.truck.step',str(output)],env=env,stdout=log,stderr=subprocess.STDOUT,check=True,timeout=120)
 comparison=json.loads(output.read_text());assert comparison['native_valid'] and comparison['truck_step_valid'] and comparison['native_solids']==comparison['truck_solids']==1
 assert abs(comparison['truck_volume']-q['expected_volume'])<1e-6,(q,comparison)
 assert max(comparison['native_to_truck_max_distance'],comparison['truck_to_native_max_distance'])<.001,comparison
 surfaces=json.loads(subprocess.check_output([str(root/'target/cylinder-angle-surface-oracle'),str(prefix)+'.truck.step'],env=env,stderr=subprocess.DEVNULL))[0]
 Path(str(prefix)+'.truck-surfaces.json').write_text(json.dumps(surfaces,indent=2)+'\n')
 planar,axial=q['bottom_planar'],q['bottom_axial'];r,height,top_radial,top_axial=q['radius'],q['height'],q['top_radial'],q['top_axial']
 import math
 def bottom_contact(p,side):return abs(p[1]-(0. if side else planar))<1e-7 and abs(p[2]-(axial if side else 0.))<1e-7
 bottom=[f for f in surfaces['faces'] if len(f['vertices'])==4 and all(bottom_contact(p,True) or bottom_contact(p,False) for p in f['vertices']) and all(any(bottom_contact(p,c) for p in f['vertices']) for c in [False,True])]
 assert len(bottom)==1,q
 expected=[(0.,0.,axial),(r,0.,axial),(0.,planar,0.),((r*r-planar*planar)**.5,planar,0.)]
 assert all(any(math.dist(p,e)<1e-7 for p in bottom[0]['vertices']) for e in expected)
 plane_errors=[abs(p[1]/planar+p[2]/axial-1) for p in bottom[0]['samples']];assert max(plane_errors)<1e-7
 bottom_errors=[e/math.hypot(1/planar,1/axial) for e in plane_errors]
 def top_contact(p,side):return abs(math.hypot(p[0],p[1])-(r if side else r-top_radial))<1e-7 and abs(p[2]-(height-top_axial if side else height))<1e-7
 top=[f for f in surfaces['faces'] if len(f['vertices'])==4 and all(top_contact(p,True) or top_contact(p,False) for p in f['vertices']) and all(any(top_contact(p,c) for p in f['vertices']) for c in [False,True])]
 assert len(top)==1,q
 top_errors=[abs(math.hypot(p[0],p[1])-(r-top_radial*(p[2]-(height-top_axial))/top_axial)) for p in top[0]['samples']]
 errors=bottom_errors+top_errors;assert max(errors)<1e-7,(q,max(errors))
 for p,h in inputs.items():assert sha(p)==h,p
 q.update(truck=result,comparison=comparison,contact_check=dict(vertices=8,sample_count=len(errors),max_surface_error=max(errors)),input_sha256=inputs)
 print(q['mode'],'Truck volume error',comparison['truck_volume']-q['expected_volume'],'sampled distance',max(comparison['native_to_truck_max_distance'],comparison['truck_to_native_max_distance']),'surface error',max(errors),flush=True)
for p,h in v['source_sha256'].items():assert sha(p)==h,p
Path('target/mixed-components-validated-summary.json').write_text(json.dumps(records,indent=2)+'\n')
