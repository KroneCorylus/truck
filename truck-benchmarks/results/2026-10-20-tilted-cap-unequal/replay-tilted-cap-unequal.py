from pathlib import Path
import json,subprocess,shutil,hashlib,os
root=Path.cwd();out=root/'target/tilted-cap-unequal-controls';out.mkdir(exist_ok=False);validation=json.load(open('truck-benchmarks/results/2026-10-20-tilted-cap-unequal/validation.json'));sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest();env=dict(os.environ,LD_LIBRARY_PATH=str(root/'target/occt-sdk/usr/lib64'));rows=[];failures=[]
for p,h in validation['source_sha256'].items():assert sha(p)==h,p
assert sha('target/release/examples/replay_occt_blend')==validation['replay_sha256']
references={str(p):sha(p) for p in Path('target/native-tilted-cap-unequal').rglob('*') if p.is_file()}
for q in json.load(open('target/native-tilted-cap-unequal/summary.json')):
 prefix=out/q['mode']
 for ext in ['input.json','input.step']:shutil.copyfile(q['prefix']+'.'+ext,str(prefix)+'.'+ext)
 before=sha(str(prefix)+'.input.step')
 with (out/(q['mode']+'.replay.log')).open('w') as log:subprocess.run(['target/release/examples/replay_occt_blend',str(prefix)],stdout=log,stderr=subprocess.STDOUT,check=True)
 report=json.load(open(str(prefix)+'.truck.json'));r=report['result'];assert r['status']=='operation-succeeded',r
 assert all(r[k] for k in ['input_unchanged','geometry_consistent','mesh_closed','step_prepared']);assert r['selected_edges']==3 and before==sha(str(prefix)+'.input.step')
 comparison=out/(q['mode']+'.comparison.json')
 with comparison.with_suffix('.log').open('w') as log:subprocess.run(['target/occt-cone-reference-plugin/compare_shapes',q['prefix']+'.result.brep',str(prefix)+'.truck.step',str(comparison)],env=env,stdout=log,stderr=subprocess.STDOUT,check=True)
 c=json.load(open(comparison));assert c['native_valid'] and c['truck_step_valid'] and c['native_solids']==c['truck_solids']==1
 difference=abs(c['truck_volume']-c['native_volume']);assert difference<c['native_volume']*.001
 distance=max(c['native_to_truck_max_distance'],c['truck_to_native_max_distance'])
 if distance>=.001:failures.append(dict(mode=q['mode'],check='distance',value=distance,threshold=.001))
 row=dict(native=q,result=r,comparison=c,volume_difference=difference,volume_difference_ppm=difference/c['native_volume']*1e6,limited_distance_pass=distance<.001);rows.append(row)
 print(q['mode'],'distance',distance,'volume difference',difference,'ppm',row['volume_difference_ppm'],flush=True)
for p,h in references.items():assert sha(p)==h,p
for p,h in validation['source_sha256'].items():assert sha(p)==h,p
assert sha('target/release/examples/replay_occt_blend')==validation['replay_sha256']
Path('target/tilted-cap-unequal-comparison-summary.json').write_text(json.dumps(dict(controls=rows,failures=failures,native_reference_sha256=references),indent=2)+'\n')
assert not failures,failures
