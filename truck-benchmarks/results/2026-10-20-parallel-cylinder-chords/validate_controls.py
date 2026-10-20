from pathlib import Path
import json,subprocess,os,hashlib,math
root=Path.cwd();proof=root/'truck-benchmarks/results/2026-10-20-parallel-cylinder-chords';raw=root/'target/native-parallel-cylinder-controls'
v=json.loads((proof/'validation.json').read_text());sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
for p,h in v['source_sha256'].items():assert sha(p)==h,p
replay=root/'target/release/examples/replay_occt_blend';assert sha(replay)==v['replay_sha256']
env=dict(os.environ,LD_LIBRARY_PATH=str(root/'target/occt-sdk/usr/lib64'))
records=json.loads((raw/'analysis.json').read_text())
for q in records:
 prefix=Path(q['prefix']);hashes={str(p):sha(p) for p in prefix.parent.glob(prefix.name+'.input.*')}
 with Path(str(prefix)+'.truck.log').open('w') as log:subprocess.run([str(replay),str(prefix)],stdout=log,stderr=subprocess.STDOUT,check=True,timeout=120)
 result=json.loads(Path(str(prefix)+'.truck.json').read_text())['result'];assert result['status']=='operation-succeeded',(q,result)
 assert result['selected_edges']==1 and all(result[k] for k in ['input_unchanged','geometry_consistent','mesh_closed','step_prepared'])
 with Path(str(prefix)+'.comparison.log').open('w') as log:subprocess.run([str(root/'target/occt-cone-reference-plugin/compare_shapes'),str(prefix)+'.result.brep',str(prefix)+'.truck.step',str(prefix)+'.comparison.json'],env=env,stdout=log,stderr=subprocess.STDOUT,check=True,timeout=120)
 compare=json.loads(Path(str(prefix)+'.comparison.json').read_text());assert compare['native_valid'] and compare['truck_step_valid'] and compare['native_solids']==compare['truck_solids']==1
 assert abs(compare['truck_volume']-q['expected_volume'])<1e-7,(q,compare)
 assert max(compare['native_to_truck_max_distance'],compare['truck_to_native_max_distance'])<.001,compare
 measured=json.loads(subprocess.check_output([str(root/'target/solid-contact-oracle'),str(prefix)+'.truck.step'],env=env,stderr=subprocess.DEVNULL))[0]
 contacts=q['expected_contacts']+[p[:2]+[q['height']] for p in q['expected_contacts']]
 error=max(min(math.dist(p,w) for w in measured['vertices']) for p in contacts);assert error<1e-7,(q,error)
 for p,h in hashes.items():assert sha(p)==h,p
 q.update(truck=result,comparison=compare,truck_measurement=measured,truck_contact_error=error,input_sha256=hashes)
 print(prefix.name,'sample',max(compare['native_to_truck_max_distance'],compare['truck_to_native_max_distance']),'volume error',compare['truck_volume']-q['expected_volume'],'contact error',error,'native input unchanged',q['native_input_unchanged'],flush=True)
for p,h in v['source_sha256'].items():assert sha(p)==h,p
(raw/'validated-summary.json').write_text(json.dumps(records,indent=2)+'\n')
print('Nine valid Truck single solids match independent contacts/material and native sampled geometry; three native inputs changed vertex tolerances.',flush=True)
