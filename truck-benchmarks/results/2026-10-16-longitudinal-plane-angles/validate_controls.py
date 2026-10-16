from pathlib import Path
import json,subprocess,os,hashlib,shutil,math
root=Path.cwd();proof=root/'truck-benchmarks/results/2026-10-16-longitudinal-plane-angles';raw=root/'target/native-longitudinal-plane-angles-final'
raw.mkdir(exist_ok=False)
v=json.loads((proof/'validation.json').read_text());sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
for p,h in v['source_sha256'].items():assert sha(p)==h,p
replay=root/'target/release/examples/replay_occt_blend';assert sha(replay)==v['replay_sha256']
plugin=root/'target/occt-cone-reference-plugin';env=dict(os.environ,LD_LIBRARY_PATH=str(root/'target/occt-sdk/usr/lib64'))
records=[]
base=root/'target/native-longitudinal-offset-angle-controls'
for q in json.loads((base/'plane-angle-proof.json').read_text()):
 q.update(source_prefix=str(base/'capture'/q['capture'].replace('.result','')),angle=30.)
 records.append(q)
for q in json.loads((root/'target/native-longitudinal-plane-extra-angles/summary.json').read_text()):
 q['source_prefix']=q.pop('prefix');records.append(q)
for i,q in enumerate(records,1):
 source=Path(q['source_prefix']);prefix=raw/f'{i:03}'
 for p in source.parent.glob(source.name+'.*'):
  if '.truck.' not in p.name and '.comparison.' not in p.name:shutil.copyfile(p,raw/(prefix.name+p.name[len(source.name):]))
 input_hashes={str(p):sha(p) for p in raw.glob(prefix.name+'.input.*')}
 with Path(str(prefix)+'.truck.log').open('w') as log:subprocess.run([str(replay),str(prefix)],stdout=log,stderr=subprocess.STDOUT,check=True,timeout=120)
 result=json.loads(Path(str(prefix)+'.truck.json').read_text())['result']
 assert result['status']=='operation-succeeded',result
 assert result['selected_edges']==1
 assert all(result[k] for k in ['input_unchanged','geometry_consistent','mesh_closed','step_prepared'])
 with Path(str(prefix)+'.comparison.log').open('w') as log:subprocess.run([str(plugin/'compare_shapes'),str(prefix)+'.result.brep',str(prefix)+'.truck.step',str(prefix)+'.comparison.json'],env=env,stdout=log,stderr=subprocess.STDOUT,check=True,timeout=120)
 compare=json.loads(Path(str(prefix)+'.comparison.json').read_text())
 assert compare['native_valid'] and compare['truck_step_valid']
 assert compare['native_solids']==compare['truck_solids']==1
 assert abs(compare['truck_volume']-q['expected_volume'])<2e-7,(q,compare)
 measured=json.loads(subprocess.check_output([str(root/'target/longitudinal-plane-angle-oracle'),str(prefix)+'.result.brep',str(prefix)+'.truck.step'],env=env))
 radius=5.;h=q['offset'];x=math.sqrt(25-h*h);sign=-1 if q['convex'] else 1
 px=x+sign*.7;alpha=math.radians(q['angle']);ux=-sign*math.cos(alpha);uy=math.sin(alpha)
 dot=px*ux+h*uy;delta=math.sqrt(dot*dot+25-px*px-h*h)
 t=min(t for t in [-dot-delta,-dot+delta] if t>0)
 expected_contacts=[[px,h,0.],[px+t*ux,h+t*uy,0.]]
 for kind,m in zip(['native','truck'],measured):
  assert len(m['bevel_planes'])==1,m
  plane=m['bevel_planes'][0];n=plane['normal'];o=plane['origin']
  angle=math.degrees(math.atan2(abs(n[0]),abs(n[1])))
  error=max(min(math.dist(p,w) for w in m['vertices']) for p in expected_contacts)
  q[kind+'_measurement']=dict(**m,angle_degrees=angle,requested_contact_max_error=error)
  if kind=='truck':
   assert abs(angle-q['angle'])<1e-8,(q,angle)
   assert error<1e-7,(q,error)
   assert max(abs(sum((p[j]-o[j])*n[j] for j in range(3))) for p in expected_contacts)<1e-7
 native_correct=abs(q['native_measurement']['angle_degrees']-q['angle'])<1e-8
 assert native_correct != (q['offset']==2.),q
 if native_correct:
  assert max(compare['native_to_truck_max_distance'],compare['truck_to_native_max_distance'])<.001
 for p,hsh in input_hashes.items():assert sha(p)==hsh,p
 q.update(prefix=str(prefix),truck=result,comparison=compare,input_sha256=input_hashes,native_requested_angle_correct=native_correct,expected_contacts=expected_contacts)
 print(prefix.name,'native-angle',q['native_measurement']['angle_degrees'],'truck-angle',q['truck_measurement']['angle_degrees'],'sample',max(compare['native_to_truck_max_distance'],compare['truck_to_native_max_distance']),'material-error',compare['truck_volume']-q['expected_volume'],flush=True)
for p,h in v['source_sha256'].items():assert sha(p)==h,p
(raw/'validated-summary.json').write_text(json.dumps(records,indent=2)+'\n')
print('All ten Truck controls preserve requested plane angle, contacts, material and valid single-solid STEP; two native angle discrepancies retained')
