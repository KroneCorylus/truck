from pathlib import Path
import json,subprocess,os
root=Path.cwd();env=dict(os.environ,LD_LIBRARY_PATH=str(root/'target/occt-sdk/usr/lib64'));base=root/'target/tilted-cap-controls';native={q['mode']:q for q in json.load(open('target/native-tilted-cap/summary.json'))};records=json.loads((base/'summary.json').read_text());failures=[]
for q in records:
 ref=native[q['mode']];assert ref['native_input_unchanged'] and q['input_unchanged'] and q['geometry_consistent'];assert q['setback_samples']==198 and q['max_setback_error']<.001
 for label in ['input','result']:
  output=base/f"{q['mode']}.{label}.comparison.json"
  with output.with_suffix('.log').open('w') as log:subprocess.run(['target/occt-cone-reference-plugin/compare_shapes',ref['prefix']+f'.{label}.brep',str(base/f"{q['mode']}.{label}.step"),str(output)],env=env,stdout=log,stderr=subprocess.STDOUT,check=True,timeout=120)
  m=json.loads(output.read_text());q[label+'_comparison']=m
  if not (m['native_valid'] and m['truck_step_valid'] and m['native_solids']==m['truck_solids']==1):failures.append((q['mode'],label,'validity'))
  if max(m['native_to_truck_max_distance'],m['truck_to_native_max_distance'])>=.001:failures.append((q['mode'],label,'distance'))
 initial=q['expected_input_volume'];assert abs(q['input_comparison']['native_volume']-initial)<1e-10 and abs(q['input_comparison']['truck_volume']-initial)<1e-10
 error=abs(q['result_comparison']['truck_volume']-q['native_reference_volume']);q.update(volume_difference=error,volume_difference_ppm=error/q['native_reference_volume']*1e6,native_reference=ref)
 if error>=q['native_reference_volume']*.001:failures.append((q['mode'],'material'))
 print(q['mode'],'volume difference',error,'ppm',q['volume_difference_ppm'],'distance',max(q['result_comparison'][k] for k in ['native_to_truck_max_distance','truck_to_native_max_distance']),'setback',q['max_setback_error'],flush=True)
Path('target/tilted-cap-comparison-summary.json').write_text(json.dumps(records,indent=2)+'\n');assert not failures,failures
