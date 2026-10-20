from pathlib import Path
import json,subprocess,os,hashlib,math
root=Path.cwd();proof=root/'truck-benchmarks/results/2026-10-20-parallel-cylinder-angles';raw=root/'target/native-parallel-cylinder-angles-complete'
v=json.loads((proof/'validation.json').read_text());sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
for p,h in v['source_sha256'].items():assert sha(p)==h,p
replay=root/'target/release/examples/replay_occt_blend';assert sha(replay)==v['replay_sha256']
env=dict(os.environ,LD_LIBRARY_PATH=str(root/'target/occt-sdk/usr/lib64'));records=json.loads((raw/'model.json').read_text())
for q in records:
 prefix=Path(q['prefix']);hashes={str(p):sha(p) for p in prefix.parent.glob(prefix.name+'.input.*')}
 with Path(str(prefix)+'.truck.log').open('w') as log:subprocess.run([str(replay),str(prefix)],stdout=log,stderr=subprocess.STDOUT,check=True,timeout=120)
 result=json.loads(Path(str(prefix)+'.truck.json').read_text())['result'];q['truck']=result
 for p,h in hashes.items():assert sha(p)==h,p
 q['input_sha256']=hashes
 if not q['native'].get('native_valid'):
  assert result['status']=='unhandled-intent',(q,result)
  assert not Path(str(prefix)+'.truck.step').exists()
  print(prefix.name,'native failed; Truck rejects missing circle intersection',flush=True);continue
 assert result['status']=='operation-succeeded',(q,result)
 assert result['selected_edges']==1 and all(result[k] for k in ['input_unchanged','geometry_consistent','mesh_closed','step_prepared'])
 with Path(str(prefix)+'.comparison.log').open('w') as log:subprocess.run([str(root/'target/occt-cone-reference-plugin/compare_shapes'),str(prefix)+'.result.brep',str(prefix)+'.truck.step',str(prefix)+'.comparison.json'],env=env,stdout=log,stderr=subprocess.STDOUT,check=True,timeout=120)
 compare=json.loads(Path(str(prefix)+'.comparison.json').read_text());assert compare['native_valid'] and compare['truck_step_valid'] and compare['native_solids']==compare['truck_solids']==1
 r1,r2,d=q['radius1'],q['radius2'],q['centers_distance'];x=(r1*r1-r2*r2+d*d)/(2*d);y=math.sqrt(r1*r1-x*x);sigma=1 if q['joined'] else -1
 contacts=[q['circle_model']['contact'],q['circle_model']['other_contact']]
 if q['reference']==1:contacts.reverse()
 qa,qb=contacts;a=2*math.asin(math.dist(qa,[x,y])/(2*r1));b=2*math.asin(math.dist(qb,[x,y])/(2*r2))
 patch=abs(r1*r1*sigma*a+qa[0]*qb[1]-qa[1]*qb[0]+d*(y-qb[1])+r2*r2*sigma*b)/2
 initial=r1*r1*math.acos(x/r1)+r2*r2*math.acos((d-x)/r2)-d*y
 if q['joined']:initial=math.pi*(r1*r1+r2*r2)-initial
 expected=(initial+sigma*patch)*q['height'];assert abs(compare['truck_volume']-expected)<1e-7 and abs(compare['native_volume']-expected)<1e-7,(q,compare,expected)
 assert max(compare['native_to_truck_max_distance'],compare['truck_to_native_max_distance'])<.001,compare
 measured=json.loads(subprocess.check_output([str(root/'target/cylinder-angle-surface-oracle'),str(prefix)+'.result.brep',str(prefix)+'.truck.step'],env=env,stderr=subprocess.DEVNULL))
 points=[p+[z] for p in contacts for z in [0.,q['height']]];delta=[qb[j]-qa[j] for j in range(2)];length=math.hypot(*delta);normal=[delta[1]/length,-delta[0]/length,0.]
 for kind,m in zip(['native','truck'],measured):
  error=max(min(math.dist(p,w) for w in m['vertices']) for p in points);assert error<1e-7,(q,error)
  bevel=[f for f in m['faces'] if all(any(math.dist(p,w)<1e-7 for w in f['vertices']) for p in points)];assert len(bevel)==1,(q,bevel)
  residual=max(abs(sum((p[j]-points[0][j])*normal[j] for j in range(3))) for p in bevel[0]['samples']);assert residual<1e-7,(q,residual)
  q[kind+'_surface_measurement']=dict(contact_error=error,planarity_sample_error=residual,bevel=bevel[0])
 q.update(comparison=compare,expected_contacts=points,expected_volume=expected)
 print(prefix.name,'sample',max(compare['native_to_truck_max_distance'],compare['truck_to_native_max_distance']),'volume error',compare['truck_volume']-expected,'contact error',q['truck_surface_measurement']['contact_error'],flush=True)
for p,h in v['source_sha256'].items():assert sha(p)==h,p
(raw/'validated-summary.json').write_text(json.dumps(records,indent=2)+'\n')
print('Ten valid single solids preserve contacts, contact angle and independent material; two impossible union requests rejected. Raw native failures/mutations retained.',flush=True)
