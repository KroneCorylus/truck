from pathlib import Path
import json,subprocess,os,hashlib,importlib.util,shutil
root=Path.cwd();proof=root/'truck-benchmarks/results/2026-10-20-shared-fillet-contours';env=dict(os.environ,LD_LIBRARY_PATH=str(root/'target/occt-sdk/usr/lib64'));sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
validation=json.loads((proof/'validation.json').read_text())
for p,h in validation['source_sha256'].items():assert sha(p)==h,p
assert sha('target/release/examples/replay_occt_blend')==validation['replay_sha256']
def module(name):
 spec=importlib.util.spec_from_file_location(name,root/'target'/f'{name}.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m
rim_check=module('check-disjoint-fillet-surfaces');cone_check=module('check-shared-cone-fillet-surfaces')
records=json.load(open('target/native-disjoint-fillet-rims/summary.json'))+[json.load(open('target/native-shared-cone-fillet/summary.json'))]
def replay(prefix):
 prefix=Path(prefix);inputs={str(p):sha(p) for p in prefix.parent.glob(prefix.name+'.input.*')}
 with Path(str(prefix)+'.truck.log').open('w') as log:subprocess.run([str(root/'target/release/examples/replay_occt_blend'),str(prefix)],stdout=log,stderr=subprocess.STDOUT,check=True,timeout=120)
 result=json.loads(Path(str(prefix)+'.truck.json').read_text())['result'];assert result['status']=='operation-succeeded',result
 assert all(result[k] for k in ['input_unchanged','geometry_consistent','mesh_closed','step_prepared']) and result['step_fitting_tolerance']==5e-5
 output=Path(str(prefix)+'.comparison.json')
 with output.with_suffix('.log').open('w') as log:subprocess.run([str(root/'target/occt-cone-reference-plugin/compare_shapes'),str(prefix)+'.result.brep',str(prefix)+'.truck.step',str(output)],env=env,stdout=log,stderr=subprocess.STDOUT,check=True,timeout=120)
 comparison=json.loads(output.read_text());assert comparison['native_valid'] and comparison['truck_step_valid'] and comparison['native_solids']==comparison['truck_solids']==1
 assert max(comparison['native_to_truck_max_distance'],comparison['truck_to_native_max_distance'])<.001,comparison
 for p,h in inputs.items():assert sha(p)==h,p
 return result,comparison,inputs
for q in records:
 assert q['native_input_unchanged'] and q['native']['native_valid'] and q['native']['solids']==1
 result,comparison,inputs=replay(q['prefix']);assert result['selected_edges']==(2 if q['mode']=='quarter' else 4)
 checks=(cone_check if q['mode']=='stack' else rim_check).check(q,Path(q['prefix']+'.truck'))
 error=abs(comparison['truck_volume']-q['expected_volume']);assert error<abs(q['expected_volume'])*1e-6 and error<checks['bevel_area_times_step_tolerance'],(q,error,checks)
 checks.update(volume_error=error,material_error_ppm=error/abs(q['expected_volume'])*1e6)
 q.update(truck=result,comparison=comparison,contact_check=checks,input_sha256=inputs)
 print(q['mode'],'volume error',error,'ppm',checks['material_error_ppm'],'sampled distance',max(comparison['native_to_truck_max_distance'],comparison['truck_to_native_max_distance']),'radius error',max(g['max_radius_error'] for g in checks['bevels']),flush=True)
case='blend/simple/I6';source=root/'target/occt-capture-mixed-chamfer-components'/case/'capture/001';prefix=root/'target/shared-fillets-I6/001';prefix.parent.mkdir(exist_ok=True)
reference={}
for pattern in ['001.input.*','001.result.*','001.capture.log']:
 for p in source.parent.glob(pattern):
  dest=prefix.parent/p.name;shutil.copy2(p,dest);assert sha(p)==sha(dest);reference[str(p)]=sha(p)
result,comparison,inputs=replay(prefix);targeted=dict(case=case,prefix=str(prefix),reference_source=str(source),reference_sha256=reference,truck=result,comparison=comparison,input_sha256=inputs)
for p,h in reference.items():assert sha(p)==h,p
print(case,'sampled distance',max(comparison['native_to_truck_max_distance'],comparison['truck_to_native_max_distance']),flush=True)
for p,h in validation['source_sha256'].items():assert sha(p)==h,p
Path('target/shared-fillets-validated-summary.json').write_text(json.dumps(dict(controls=records,targeted=targeted),indent=2)+'\n')
