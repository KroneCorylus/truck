from pathlib import Path
import json,subprocess,os,math,hashlib
root=Path.cwd();raw=root/'target/native-convex-corner-clip';proof=root/'truck-benchmarks/results/2026-10-20-convex-corner-clipping'
env=dict(os.environ,LD_LIBRARY_PATH=str(root/'target/occt-sdk/usr/lib64'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
v=json.loads((proof/'validation.json').read_text())
for p,h in v['source_sha256'].items():assert sha(p)==h,p
assert sha('target/release/examples/replay_occt_blend')==v['replay_sha256']
records=json.loads((raw/'analysis.json').read_text())
for q in records:
 assert q['native_input_unchanged'] and q['native']['native_valid'] and q['native']['solids']==1
 prefix=Path(q['prefix']);inputs={str(p):sha(p) for p in prefix.parent.glob(prefix.name+'.input.*')}
 with Path(str(prefix)+'.truck.log').open('w') as log:subprocess.run([str(root/'target/release/examples/replay_occt_blend'),str(prefix)],stdout=log,stderr=subprocess.STDOUT,check=True,timeout=120)
 result=json.loads(Path(str(prefix)+'.truck.json').read_text())['result'];assert result['status']=='operation-succeeded',result
 assert result['selected_edges']==3 and all(result[k] for k in ['input_unchanged','geometry_consistent','mesh_closed','step_prepared'])
 output=Path(str(prefix)+'.comparison.json')
 with output.with_suffix('.log').open('w') as log:subprocess.run([str(root/'target/occt-cone-reference-plugin/compare_shapes'),str(prefix)+'.result.brep',str(prefix)+'.truck.step',str(output)],env=env,stdout=log,stderr=subprocess.STDOUT,check=True,timeout=120)
 comparison=json.loads(output.read_text());assert comparison['native_valid'] and comparison['truck_step_valid'] and comparison['native_solids']==comparison['truck_solids']==1
 assert abs(comparison['truck_volume']-q['expected_volume'])<1e-9,comparison
 surfaces={}
 for kind,path in [('native',str(prefix)+'.result.brep'),('truck',str(prefix)+'.truck.step')]:
  surfaces[kind]=json.loads(subprocess.check_output([str(root/'target/cylinder-angle-surface-oracle'),path],env=env,stderr=subprocess.DEVNULL))[0]
  Path(str(prefix)+f'.{kind}-surfaces.json').write_text(json.dumps(surfaces[kind],indent=2)+'\n')
 vertices=surfaces['truck']['vertices'];expected=q['halfspace_vertices']
 errors=[max(min(math.dist(p,w) for w in b) for p in a) for a,b in [(vertices,expected),(expected,vertices)]];assert max(errors)<1e-8,errors
 plane_checks=[]
 for expected_face in q['halfspace_faces']:
  candidates=[f for f in surfaces['truck']['faces'] if all(any(math.dist(p,w)<1e-8 for w in f['vertices']) for p in expected_face['vertices'])]
  assert len(candidates)==1,candidates
  n,d=expected_face['normal'],expected_face['offset'];residual=max(abs(sum(n[i]*p[i] for i in range(3))-d) for p in candidates[0]['samples']);assert residual<1e-8,residual
  plane_checks.append(residual)
 for p,h in inputs.items():assert sha(p)==h,p
 q.update(truck=result,comparison=comparison,vertex_errors=errors,plane_sample_errors=plane_checks,input_sha256=inputs)
 print(prefix.name,'Truck volume error',comparison['truck_volume']-q['expected_volume'],'native error',q['native_volume_error'],'raw sampled distance',max(comparison['native_to_truck_max_distance'],comparison['truck_to_native_max_distance']),flush=True)
for p,h in v['source_sha256'].items():assert sha(p)==h,p
Path('target/convex-corner-validated-summary.json').write_text(json.dumps(records,indent=2)+'\n')
