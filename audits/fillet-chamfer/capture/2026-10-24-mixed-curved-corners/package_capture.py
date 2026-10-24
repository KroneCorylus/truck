from pathlib import Path
from collections import Counter
import json,hashlib,math,zipfile,re
source=Path('target/occt-capture-mixed-curved-corners');previous=Path('target/occt-capture-torus-corners');dest=Path('audits/fillet-chamfer/capture/2026-10-24-mixed-curved-corners')
new=json.loads((source/'results.json').read_text());old=json.loads(Path('audits/fillet-chamfer/capture/2026-10-22-torus-corners/results.json').read_text());sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
assert not new['kernel_tracked_changes'] and new['test_data_verified']
validation=json.loads(Path('truck-benchmarks/results/2026-10-22-mixed-curved-corners/validation.json').read_text())
for path,digest in validation['source_sha256'].items():
 assert sha(path)==digest,path
assert sha('target/release/examples/replay_occt_blend')==validation['replay_sha256']
old_artifacts={path:digest for case in old['cases'] for path,digest in case['artifacts'].items()}
for case in new['cases']:
 for path,digest in case['source_sha256'].items():
  assert sha(Path('../investigations/sequential-holes-2026-10-10/occt')/path)==digest,path

for name,h in new['library_sha256'].items():
 assert sha(Path('target/occt-sdk/usr/lib64')/name)==h,name
for name,h in new['harness_sha256'].items():
 assert sha(Path('../investigations/sequential-holes-2026-10-10/occt')/name)==h,name
assert new['workers']==1 and new['timeout_seconds']==120
assert len(new['cases'])==396
assert len({c['id'] for c in new['cases']})==396
assert sum(len(c['captures']) for c in new['cases'])==648
assert all(old[k]==new[k] for k in ['library_sha256','harness_sha256'])
assert {k:v for k,v in old['test_data'].items() if k!='archive_members'}==new['test_data']
assert Counter(c['native']['status'] for c in new['cases'])=={'passed':396}
for field in ['kernel_source_sha256','source_sha256','executable_sha256']:
 for p,h in new[field].items():assert sha(p)==h,p
flatten=lambda m:{(c['id'],q['prefix']):q for c in m['cases'] for q in c['captures']}
a,b=flatten(old),flatten(new);assert set(a)==set(b) and len(b)==648
status=lambda q:(q.get('truck') or {}).get('result',{}).get('status',q.get('status'))
def checks(q):
 r=(q.get('truck') or {}).get('result',{});m=q.get('sampled_comparison') or {}
 distances=[m.get(k) for k in ['native_to_truck_max_distance','truck_to_native_max_distance']]
 return dict(constructed=status(q)=='operation-succeeded',**{k:r.get(k) is True for k in ['input_unchanged','geometry_consistent','mesh_closed','step_prepared']},native_valid=m.get('native_valid') is True,truck_step_valid=m.get('truck_step_valid') is True,single_solid=m.get('native_solids')==m.get('truck_solids')==1,sampled_distance=all(isinstance(d,(int,float)) and math.isfinite(d) and 0<=d<=.001 for d in distances))
def same_curve(a,b):
 if a.get('curve_type')!=b.get('curve_type'):return False
 x,y=a['points'],b['points']
 return len(x)==len(y) and any(all(math.dist(p,q)<1e-12 for p,q in zip(x,z)) for z in [y,list(reversed(y))])
def normalized_step(path):
 # OCCT writes capture time in the FILE_NAME header; compare every other byte.
 return re.sub(rb"(FILE_NAME\('Open CASCADE Shape Model',')\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(')", rb"\1CAPTURE_TIME\2", path.read_bytes())
native_geometry_files=0
reference_samples=[];changes=[];numeric=[];scope_changes=[];regressions=[];changed_scope_losses=[];seed_changes=[];native_changes=[];exact=0;unchanged_intent=0;scope_counts=Counter()
for key,q in b.items():
 prior=a[key];tag=dict(case=key[0],capture=key[1])
 new_task=json.loads((source/key[0]/'capture'/f'{key[1]}.input.json').read_text())
 old_task_path=Path(key[0])/'capture'/f'{key[1]}.input.json'
 assert sha(previous/old_task_path)==old_artifacts[str(old_task_path)],old_task_path
 old_task=json.loads((previous/old_task_path).read_text())
 assert new_task==old_task,key
 assert prior['task']==q['task'],key
 for suffix in ['input.brep','result.brep','input.step','result.step']:
  old_file=previous/key[0]/'capture'/f'{key[1]}.{suffix}'
  new_file=source/key[0]/'capture'/f'{key[1]}.{suffix}'
  assert old_file.exists()==new_file.exists(),(key,suffix)
  if old_file.exists():
   assert sha(old_file)==old_artifacts[str(old_file.relative_to(previous))],old_file
   left,right=(normalized_step(old_file),normalized_step(new_file)) if suffix.endswith('step') else (old_file.read_bytes(),new_file.read_bytes())
   assert left==right,(key,suffix,'native geometry changed')
   native_geometry_files+=1
 if status(prior)=='operation-succeeded':
  assert prior['truck']==q.get('truck'),key
  assert prior.get('sampled_comparison')==q.get('sampled_comparison'),key
 assert set(new_task['stored_tolerances'])=={'face','edge','vertex'}
 assert all(math.isfinite(v) and v>=0 for v in new_task['stored_tolerances'].values())
 changed=False
 reported_scope=(q.get('truck') or {}).get('result',{}).get('selection_scope')
 assert reported_scope=='native-contours',(key,reported_scope)
 scope_counts[new_task['operation']+'-'+reported_scope]+=1
 unchanged_intent+=not changed
 if prior.get('native_result')!=q.get('native_result'):native_changes.append(dict(**tag,before=prior.get('native_result'),after=q.get('native_result')))
 if status(prior)!=status(q):changes.append(dict(**tag,changed_scope=changed,before=status(prior),after=status(q)))
 c0,c1=checks(prior),checks(q)
 if status(prior)=='operation-succeeded' and any(c0[k] and not c1[k] for k in c0):
  (changed_scope_losses if changed else regressions).append(dict(**tag,before=c0,after=c1,status=status(q)))
 if status(prior)==status(q)=='operation-succeeded':
  r0=prior['truck']['result'];r=q['truck']['result']
  m0=prior.get('sampled_comparison') or {};m=(q.get('sampled_comparison') or {})
  diff={k:dict(before=r0.get(k),after=r.get(k)) for k in ['mesh_volume','selected_edges'] if r0.get(k)!=r.get(k)}
  diff.update({k:dict(before=m0.get(k),after=m.get(k)) for k in ['truck_volume','native_to_truck_max_distance','truck_to_native_max_distance'] if m0.get(k)!=m.get(k)})
  if diff:numeric.append(dict(**tag,changed_scope=changed,changes=diff))
  exact+=int(r0==r and m0==m)
summary=dict(kernel_revision=new['kernel_revision'],native=dict(Counter(c['native']['status'] for c in new['cases'])),captures=len(b),before_statuses=dict(Counter(status(q) for q in a.values())),after_statuses=dict(Counter(status(q) for q in b.values())),capture_scopes=dict(scope_counts),unchanged_intent_records=unchanged_intent,successful_checks=dict(Counter(k for q in b.values() if status(q)=='operation-succeeded' for k,v in checks(q).items() if v)),limited_comparison_passes=sum(all(checks(q).values()) for q in b.values()),status_changes=changes,scope_changes=scope_changes,changed_scope_check_losses=changed_scope_losses,unchanged_intent_regressions=regressions,numeric_changes=numeric,exact_successful_results=exact,seed_capture_changes=seed_changes,native_reference_changes=native_changes)
tolerance_summary=dict(threshold=.001,inputs=[],native_results=[],truck_step=[],native_result_count=0,truck_step_count=0)
for key,q in b.items():
 tag=dict(case=key[0],capture=key[1])
 sources=[('inputs',q['task'].get('stored_tolerances')),('native_results',q.get('native_result',{}).get('stored_tolerances')),('truck_step',(q.get('sampled_comparison') or {}).get('truck_step_tolerances'))]
 for label,values in sources:
  if values is None:continue
  assert set(values)=={'face','edge','vertex'} and all(math.isfinite(v) and v>=0 for v in values.values())
  if label=='native_results':tolerance_summary['native_result_count']+=1
  if label=='truck_step':tolerance_summary['truck_step_count']+=1
  if max(values.values())>.001:tolerance_summary[label].append(dict(**tag,tolerances=values,status=status(q),limited_comparison=all(checks(q).values())))
summary['stored_tolerances']=tolerance_summary
summary['identical_native_geometry_files_excluding_step_capture_timestamp']=native_geometry_files
summary['added_reference_samples']=reference_samples
summary['input_scope_counts']=dict(Counter('no-solid' if q['task'].get('solids')==0 else 'solid' for q in b.values()))
summary['matched_native_rejections']=[dict(case=k[0],capture=k[1]) for k,q in b.items() if status(q)=='operation-failed' and q.get('native_result',{}).get('status')=='operation-failed']
summary['positive_solid_reference_count']=sum(q['task'].get('solids')==1 and q.get('native_result',{}).get('native_valid') is True and q.get('native_result',{}).get('solids')==1 for q in b.values())
summary['unchanged_failure_diagnostic_changes']=[dict(case=key[0],capture=key[1],before=a[key].get('truck'),after=q.get('truck')) for key,q in b.items() if status(a[key])==status(q)!='operation-succeeded' and a[key].get('truck')!=q.get('truck')]

Path('target/mixed-curved-corner-comparison-provisional.json').write_text(json.dumps(summary,indent=2)+'\n')
assert not reference_samples,reference_samples
assert not seed_changes,seed_changes
assert not native_changes,native_changes
assert not regressions,regressions
positive_keys={k for k,q in b.items() if q['task'].get('solids')==1 and q.get('native_result',{}).get('native_valid') is True and q.get('native_result',{}).get('solids')==1}
limited_keys={k for k,q in b.items() if all(checks(q).values())}
assert limited_keys<=positive_keys
summary['unresolved_positive_solid_operations']=[dict(case=k[0],capture=k[1],status=status(b[k])) for k in sorted(positive_keys-limited_keys)]
summary['new_limited_comparison_passes']=[dict(case=k[0],capture=k[1]) for k,q in b.items() if all(checks(q).values()) and not all(checks(a[k]).values())]
artifacts={p:(source/p,h) for case in new['cases'] for p,h in case['artifacts'].items()}
artifacts['runtime.log']=(source/'runtime.log',sha(source/'runtime.log'))
new['test_data']['archive_members']={}
for i,(path,h) in enumerate(sorted(new['test_data']['external_files'].items())):
 member=f'data/{i:02}-{Path(path).name}';new['test_data']['archive_members'][path]=member;artifacts[member]=(Path(path),h)
p=Path('target/chamfer-public-data-provenance.json');artifacts['data/provenance.json']=(p,sha(p))
dest.mkdir(parents=True,exist_ok=False);archive=dest/'artifacts.zip'
with zipfile.ZipFile(archive,'w',zipfile.ZIP_DEFLATED) as z:
 for member,(p,h) in sorted(artifacts.items()):assert sha(p)==h,p;z.write(p,member)
with zipfile.ZipFile(archive) as z:
 assert set(z.namelist())==set(artifacts)
 for member,(p,h) in artifacts.items():assert hashlib.sha256(z.read(member)).hexdigest()==h==sha(p),p
for field in ['kernel_source_sha256','source_sha256','executable_sha256']:
 for p,h in new[field].items():assert sha(p)==h,p
new['artifact_archive']='artifacts.zip';new['artifact_archive_sha256']=sha(archive)
(dest/'results.json').write_text(json.dumps(new,indent=2)+'\n');(dest/'comparison-summary.json').write_text(json.dumps(summary,indent=2)+'\n')
print(json.dumps({k:v for k,v in summary.items() if k not in ['unchanged_failure_diagnostic_changes','stored_tolerances','added_reference_samples']},indent=2));print('Verified',len(artifacts),'archive members',sha(archive))
