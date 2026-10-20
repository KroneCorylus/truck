from pathlib import Path
import json,subprocess,os,hashlib,math
root=Path.cwd();proof=root/'truck-benchmarks/results/2026-10-20-cylindrical-terminal-fans';sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
v=json.loads((proof/'validation.json').read_text())
for p,h in v['source_sha256'].items():assert sha(p)==h,p
replay=root/'target/release/examples/replay_occt_blend';assert sha(replay)==v['replay_sha256']
env=dict(os.environ,LD_LIBRARY_PATH=str(root/'target/occt-sdk/usr/lib64'));records=[]
for kind in ['fan','unsplit']:
 raw=root/f'target/native-cylindrical-terminal-{kind}'
 for i,q in enumerate(json.loads((raw/'analysis.json').read_text())):
  prefix=Path(q['prefix']);hashes={str(p):sha(p) for p in prefix.parent.glob(prefix.name+'.input.*')}
  with Path(str(prefix)+'.truck.log').open('w') as log:subprocess.run([str(replay),str(prefix)],stdout=log,stderr=subprocess.STDOUT,check=True,timeout=120)
  result=json.loads(Path(str(prefix)+'.truck.json').read_text())['result'];assert result['status']=='operation-succeeded',(q,result)
  assert result['selected_edges']==q['count'] and all(result[k] for k in ['input_unchanged','geometry_consistent','mesh_closed','step_prepared'])
  for p,h in hashes.items():assert sha(p)==h,p
  reference=root/f'target/native-cylindrical-terminal-unsplit/capture/{i+1:03}.result.brep'
  comparisons={}
  for label,native in [('raw',Path(str(prefix)+'.result.brep')),('equivalent_unsplit',reference)]:
   output=Path(str(prefix)+f'.{label}-comparison.json')
   with output.with_suffix('.log').open('w') as log:subprocess.run([str(root/'target/occt-cone-reference-plugin/compare_shapes'),str(native),str(prefix)+'.truck.step',str(output)],env=env,stdout=log,stderr=subprocess.STDOUT,check=True,timeout=120)
   comparisons[label]=json.loads(output.read_text())
  compare=comparisons['equivalent_unsplit'];assert compare['native_valid'] and compare['truck_step_valid'] and compare['native_solids']==compare['truck_solids']==1
  assert abs(compare['truck_volume']-q['expected_volume'])<1e-7,(q,compare)
  assert max(compare['native_to_truck_max_distance'],compare['truck_to_native_max_distance'])<.001,compare
  measured=json.loads(subprocess.check_output([str(root/'target/cylinder-angle-surface-oracle'),str(prefix)+'.truck.step'],env=env,stderr=subprocess.DEVNULL))[0]
  r,h,L,dy,dz=q['radius'],q['halfwidth'],q['level'],q['planar_distance'],q['wall_distance'];contacts=[];bevel_checks=[]
  for sign in [1.,-1.][:q['count']]:
   points=[[0.,sign*h,L-dz],[math.sqrt(r*r-h*h),sign*h,L-dz],[0.,sign*(h-dy),L],[math.sqrt(r*r-(h-dy)**2),sign*(h-dy),L]];contacts+=points
   bevel=[f for f in measured['faces'] if all(any(math.dist(p,w)<1e-7 for w in f['vertices']) for p in points)];assert len(bevel)==1,(q,bevel)
   normal=[0.,-sign*dz/dy,-1.];length=math.sqrt(sum(n*n for n in normal));normal=[n/length for n in normal]
   residual=max(abs(sum((p[j]-points[0][j])*normal[j] for j in range(3))) for p in bevel[0]['samples']);assert residual<1e-7,(q,residual)
   bevel_checks.append(dict(contacts=points,normal=normal,plane_sample_error=residual))
  error=max(min(math.dist(p,w) for w in measured['vertices']) for p in contacts);assert error<1e-7,(q,error)
  q.update(truck=result,comparisons=comparisons,contact_error=error,bevel_checks=bevel_checks,input_sha256=hashes);records.append(q)
  print(kind,i+1,'raw native valid',comparisons['raw']['native_valid'],'raw sample',max(comparisons['raw']['native_to_truck_max_distance'],comparisons['raw']['truck_to_native_max_distance']),'equivalent sample',max(compare['native_to_truck_max_distance'],compare['truck_to_native_max_distance']),'volume error',compare['truck_volume']-q['expected_volume'],flush=True)
for p,h in v['source_sha256'].items():assert sha(p)==h,p
(root/'target/cylindrical-terminal-validated-summary.json').write_text(json.dumps(records,indent=2)+'\n')
print('Twelve Truck controls preserve analytic material and contacts and match valid unsplit native controls; six raw subdivided native discrepancies retained.',flush=True)
