from pathlib import Path
from collections import Counter
import hashlib,json,math,zipfile
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
paths=[Path('audits/fillet-chamfer/capture')/n/'results.json' for n in ['2026-10-24-exact-knots','2026-10-24-nine-case-cohort']]
ms=[json.loads(p.read_text()) for p in paths]
for f in ['kernel_source_sha256','source_sha256','executable_sha256','library_sha256','harness_sha256']:
 assert ms[0][f]==ms[1][f],f
assert ms[0]['test_data']['external_files']==ms[1]['test_data']['external_files']
expected={'blend/complex/D2':[10,5,10,5,5,10,5,10,5,5], 'blend/simple/Q9':[2.5], 'blend/simple/R1':[11]*3,'blend/simple/R2':[11], 'blend/simple/R3':[.5], 'blend/simple/R5':[.5], 'blend/simple/R6':[.5], 'blend/simple/U9':[150], 'blend/simple/V7':[50]}
intent=[]
with zipfile.ZipFile(paths[1].parent/'artifacts.zip') as z:
 begin=z.read('native-source/tests/blend/begin').decode()
 assert 'dset SCALE 1000' in begin and 'dset SCALE1 10' in begin
 for c in ms[1]['cases']:
  script=z.read('native-source/tests/'+c['id'])
  assert hashlib.sha256(script).hexdigest()==c['source_sha256']['tests/'+c['id']]
  commands=[line.strip() for line in script.decode().splitlines() if line.strip().startswith('blend ')]
  assert len(commands)==1 and len(c['captures'])==1 and c['captures'][0]['prefix']=='001'
  q=c['captures'][0]; raw=z.read(c['id']+'/capture/001.input.json');t=json.loads(raw)
  assert t['operation']=='fillet' and [s['radius'] for s in t['selection']]==expected[c['id']]
  contours={v['index']:v for v in t['contours']}
  selections=[]
  for s in t['selection']:
   co=contours[s['contour_index']];assert s['radius']==co['constant_radius']
   def same(e):return e['curve_type']==s['curve_type'] and (e['points']==s['points'] or e['points']==list(reversed(s['points'])))
   matches=[i for i,e in enumerate(co['edges']) if same(e)]
   assert matches,(c['id'],'seed not present in native contour')
   assert all(math.isfinite(v) for p in s['points'] for v in p)
   selections.append(dict(radius=s['radius'],contour=s['contour_index'],curve_type=s['curve_type'],point_count=len(s['points']),start=s['points'][0],end=s['points'][-1],matching_contour_edge_indices=matches))
  assert q['truck']['result']['selection_scope']=='native-contours'
  intent.append(dict(case=c['id'],capture='001',command=commands[0],script_sha256=hashlib.sha256(script).hexdigest(),raw_task_sha256=hashlib.sha256(raw).hexdigest(),operation=t['operation'],selections=selections,contours=[dict(index=k,radius=v['constant_radius'],edge_count=len(v['edges'])) for k,v in contours.items()],input_solids=t['solids'],native_result_solids=q['native_result']['solids'],native_result_valid=q['native_result']['native_valid'],selection_scope='native-contours'))
def ck(q):
 r=(q.get('truck') or {}).get('result',{});v=q.get('sampled_comparison') or {};ds=[v.get(k) for k in ['native_to_truck_max_distance','truck_to_native_max_distance']]
 return r.get('status')=='operation-succeeded' and all(r.get(k) is True for k in ['input_unchanged','geometry_consistent','mesh_closed','step_prepared']) and v.get('native_valid') is True and v.get('truck_step_valid') is True and v.get('native_solids')==v.get('truck_solids')==1 and all(isinstance(d,(int,float)) and math.isfinite(d) and 0<=d<=.001 for d in ds)
sets=[];rows=[];refs=[]
for i,(p,m) in enumerate(zip(paths,ms)):
 keys=set()
 for c in m['cases']:
  for q in c['captures']:
   key=(c['id'],q['prefix']);assert key not in keys;keys.add(key)
   pos=q['task'].get('solids')==1 and q.get('native_result',{}).get('native_valid') is True and q.get('native_result',{}).get('solids')==1
   rows.append(dict(case=key[0],capture=key[1],cohort=i,status=q.get('truck',{}).get('result',{}).get('status',q.get('status')),positive_solid_reference=pos,limited_pass=ck(q),input_no_solid=q['task'].get('solids')==0))
 sets.append(keys)
 assert sha(p.parent/'artifacts.zip')==m['artifact_archive_sha256']
 refs.append(dict(manifest=str(p),manifest_sha256=sha(p),archive=str(p.parent/'artifacts.zip'),archive_sha256=m['artifact_archive_sha256'],case_count=len(m['cases']),operation_count=len(keys),root_revision=m['kernel_revision'],started_utc=m['started_utc']))
assert sets[0].isdisjoint(sets[1])
case_sets=[{c['id'] for c in m['cases']} for m in ms];assert case_sets[0].isdisjoint(case_sets[1])
counts=Counter(r['status'] for r in rows);positive=sum(r['positive_solid_reference'] for r in rows);passed=sum(r['limited_pass'] for r in rows)
index=dict(scope_kind='combined-immutable-capture-evidence-index',not_a_fresh_combined_run=True,implementation_revision=ms[0]['kernel_revision'],cohorts=refs,case_count=len(set.union(*case_sets)),operation_count=len(rows),statuses=dict(counts),positive_solid_reference_count=positive,limited_comparison_passes=passed,constructions=counts['operation-succeeded'],unresolved_positive_solid_operations=sum(r['positive_solid_reference'] and not r['limited_pass'] for r in rows),constructed_discrepancies=sum(r['status']=='operation-succeeded' and not r['limited_pass'] for r in rows),no_solid_inputs=sum(r['input_no_solid'] for r in rows),new_cohort_passes=sum(r['limited_pass'] for r in rows if r['cohort']==1),common_identity_dictionaries={f:ms[0][f] for f in ['kernel_source_sha256','source_sha256','executable_sha256','library_sha256','harness_sha256']},native_intent_review=intent,operations=rows,limitations=['Combined evidence across two executions, not a fresh 405-case run.','Physical selection review checks captured native-contour geometry and seeds against archived commands; it does not re-execute native edge numbering.','Shell capture outcomes are not solid capability passes.','No global distance guarantee or newly resolved previous operation is inferred.'])
assert (index['case_count'],index['operation_count'],positive,passed,index['unresolved_positive_solid_operations'])==(405,657,560,438,122)
Path('target/nine-case-union-index-2026-10-24.json').write_text(json.dumps(index,indent=2)+'\n')
print({k:v for k,v in index.items() if k not in ['cohorts','common_identity_dictionaries','native_intent_review','operations']})
