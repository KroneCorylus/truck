import json,math,hashlib,re,zipfile,collections
from pathlib import Path
out=Path('target/d-exact-capture-review/final');base=Path('audits/fillet-chamfer/capture/2026-10-24-mixed-curved-corners');live=Path('target/occt-capture-exact-knots-2026-10-24')
p=json.loads((out/'snapshot.json').read_bytes());old=json.loads((base/'results.json').read_bytes());sha=lambda b:hashlib.sha256(b).hexdigest(); flatten=lambda m:{(c['id'],q['prefix']):q for c in m['cases'] for q in c['captures']};a,b=flatten(old),flatten(p)
status=lambda q:(q.get('truck') or {}).get('result',{}).get('status',q.get('status'))
def checks(q):
 r=(q.get('truck') or {}).get('result',{});m=q.get('sampled_comparison') or {};ds=[m.get(k) for k in ['native_to_truck_max_distance','truck_to_native_max_distance']]
 return dict(constructed=status(q)=='operation-succeeded',**{k:r.get(k) is True for k in ['input_unchanged','geometry_consistent','mesh_closed','step_prepared']},native_valid=m.get('native_valid') is True,truck_step_valid=m.get('truck_step_valid') is True,single_solid=m.get('native_solids')==m.get('truck_solids')==1,sampled_distance=all(isinstance(d,(int,float)) and math.isfinite(d) and 0<=d<=.001 for d in ds))
def diff(x,y):return {k:dict(before=x.get(k),after=y.get(k)) for k in x.keys()|y.keys() if x.get(k)!=y.get(k)}
rows=[];oldcases={c['id']:c for c in old['cases']};verified=0;mismatches=[]
norm=lambda x:re.sub(rb"(FILE_NAME\('Open CASCADE Shape Model',')\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(')",rb'\1CAPTURE_TIME\2',x)
with zipfile.ZipFile(base/'artifacts.zip') as z:
 for c in p['cases']:
  for q in c['captures']:
   k=c['id'],q['prefix'];prior=a[k]
   for suffix in ['input.brep','result.brep','input.step','result.step','input.json','result.json']:
    f=f'{k[0]}/capture/{k[1]}.{suffix}';expected=oldcases[k[0]]['artifacts'].get(f);current=c['artifacts'].get(f)
    if expected is None and current is None:continue
    if expected is None or current is None:mismatches.append([f,'presence']);continue
    l=z.read(f);r=(live/f).read_bytes();assert sha(l)==expected and sha(r)==current,f
    if (norm(l) if suffix.endswith('step') else l)!=(norm(r) if suffix.endswith('step') else r):mismatches.append([f,'contents'])
    verified+=1
   row=dict(id='/'.join(k),before=status(prior),after=status(q),checks_before=checks(prior),checks_after=checks(q),task_equal=prior['task']==q['task'],native_result_equal=prior.get('native_result')==q.get('native_result'),truck_changes=diff(prior['truck']['result'],q['truck']['result']),comparison_changes=diff(prior.get('sampled_comparison') or {},q.get('sampled_comparison') or {}))
   rows.append(row)
report=dict(provisional=True,scope='Single frozen incomplete snapshot; no final corpus totals or baseline reconciliation',snapshot_sha256=sha((out/'snapshot.json').read_bytes()),baseline_sha256=sha((base/'results.json').read_bytes()),kernel_revision=p['kernel_revision'],completed_native_cases=len(p['cases']),completed_operations=len(rows),native_statuses=dict(collections.Counter(c['native']['status'] for c in p['cases'])),statuses=dict(collections.Counter(r['after'] for r in rows)),status_changes=[r for r in rows if r['before']!=r['after']],previous_success_changes=[r for r in rows if r['before']=='operation-succeeded' and (r['truck_changes'] or r['comparison_changes'])],canonical_check_losses=[r for r in rows if any(v and not r['checks_after'][k] for k,v in r['checks_before'].items())],limited_passes_in_completed_subset=sum(all(r['checks_after'].values()) for r in rows),limited_passes_before_same_subset=sum(all(r['checks_before'].values()) for r in rows),native_files_hash_verified=verified,native_files_changed_except_step_time=mismatches,runtime_library_manifest_equal=p['library_sha256']==old['library_sha256'],harness_manifest_equal=p['harness_sha256']==old['harness_sha256'],test_data_verified=p['test_data_verified'],rows=rows)
(out/'review.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({k:v for k,v in report.items() if k not in ['rows','previous_success_changes']},indent=2))
for r in report['previous_success_changes']:print(r['id'],json.dumps(r['truck_changes']),json.dumps(r['comparison_changes']))
