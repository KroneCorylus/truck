from pathlib import Path
import json,hashlib,subprocess,os,math
root=Path.cwd();proof=root/'truck-benchmarks/results/2026-10-16-cone-contact-oracle';sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
manifest=json.loads((root/'target/occt-capture-cone-references/results.json').read_text());baseline=root/'target/occt-capture-cone-references'
for group in ['kernel_source_sha256','source_sha256','executable_sha256']:
 for p,h in manifest[group].items():assert sha(p)==h,p
sdk=root/'target/occt-sdk/usr/lib64';plugin=root/'target/occt-cone-reference-plugin';env=dict(os.environ,LD_LIBRARY_PATH=str(sdk));replay=root/'target/release/examples/replay_occt_blend'
controls=[]
for folder in ['native-cone45-reference-probe','native-cone45-swapped-probe']:
 raw=root/'target'/folder
 for p in sorted((raw/'capture').glob('*.input.json')):
  prefix=str(p)[:-len('.input.json')];task=json.loads(p.read_text());chosen=task['selection'][0];index=int(p.name[:3])-1
  widening=index>=4;slope=1. if widening else -1.;top=5. if widening else 3.;g=math.sqrt(2.)
  cone=chosen.get('reference_cone') is not None
  if chosen['mode']=='two-distances':d0,d1=chosen['distances']
  else:
   d0=chosen['distance'];tan=math.tan(math.radians(chosen['angle_degrees']));d1=d0*tan*g/(1+slope*tan)
  radial,along=(d1,d0) if cone else (d0,d1);vertical=along/g
  expected=math.pi*2*49/3-math.pi*(top*radial*vertical-slope*radial*vertical*vertical/3-radial*radial*vertical/3)
  inputs={str(q):sha(q) for q in p.parent.glob(p.name[:3]+'.input.*')}
  with Path(prefix+'.truck.log').open('w') as log:subprocess.run([str(replay),prefix],stdout=log,stderr=subprocess.STDOUT,check=True,timeout=120)
  r=json.loads(Path(prefix+'.truck.json').read_text())['result'];assert r['status']=='operation-succeeded' and r['selected_edges']==2,r
  assert all(r[k] for k in ['input_unchanged','geometry_consistent','mesh_closed','step_prepared'])
  with Path(prefix+'.comparison.log').open('w') as log:subprocess.run([str(plugin/'compare_shapes'),prefix+'.result.brep',prefix+'.truck.step',prefix+'.comparison.json'],env=env,stdout=log,stderr=subprocess.STDOUT,check=True,timeout=120)
  comparison=json.loads(Path(prefix+'.comparison.json').read_text());assert comparison['native_valid'] and comparison['truck_step_valid'];assert comparison['native_solids']==comparison['truck_solids']==1
  assert abs(comparison['truck_volume']-expected)<1e-5,(prefix,comparison['truck_volume'],expected)
  mismatch=folder=='native-cone45-swapped-probe' and index in [4,6]
  if mismatch:assert abs(comparison['native_volume']-expected)>1
  else:assert abs(comparison['native_volume']-expected)<1e-7
  for q,h in inputs.items():assert sha(q)==h,q
  controls.append(dict(folder=folder,index=index,prefix=prefix,expected_plane=radial,expected_cone=along,expected_volume=expected,native_setback_mismatch=mismatch,input_sha256=inputs,comparison=comparison,truck=r))
  print(folder,p.name[:3],'native error',comparison['native_volume']-expected,'truck error',comparison['truck_volume']-expected,flush=True)
rows=[];source_artifacts={}
for c in manifest['cases']:
 for q in c['captures']:
  if c['id'] not in ['chamfer/dist_dist/'+case for case in ['C1','C6','C7','D1','D2','D5','D6','D9','E1']]:continue
  prefix=baseline/c['id']/'capture'/q['prefix'];task=json.loads(Path(str(prefix)+'.input.json').read_text());seed=task['selection'][0];points=seed['points'];d0,d1=seed['distances'];plane,cone=(d1,d0) if seed.get('reference_cone') else (d0,d1)
  for relative,h in c['artifacts'].items():
   if '/capture/'+q['prefix']+'.' in relative:
    path=baseline/relative;assert sha(path)==h,path;source_artifacts[relative]=h
  for kind in ['native','truck']:
   rows.append(dict(case=c['id'],capture=q['prefix'],kind=kind,input=str(prefix)+'.input.brep',result=str(prefix)+('.result.brep' if kind=='native' else '.truck.step'),points=[points[0],points[len(points)//2],points[-1]],limit=2*max(d0,d1),expected_plane=plane,expected_cone=cone,reference='cone' if seed.get('reference_cone') else 'plane',d0=d0,d1=d1,native_setback_mismatch=True))
for c in controls:
 task=json.loads(Path(c['prefix']+'.input.json').read_text());seed=task['selection'][0];points=seed['points']
 for kind in ['native','truck']:
  rows.append(dict(case=c['folder'],capture=f"{c['index']+1:03}",kind=kind,input=c['prefix']+'.input.brep',result=c['prefix']+('.result.brep' if kind=='native' else '.truck.step'),points=[points[0],points[len(points)//2],points[-1]],limit=2*max(c['expected_plane'],c['expected_cone']),expected_plane=c['expected_plane'],expected_cone=c['expected_cone'],reference='cone' if seed.get('reference_cone') else 'plane',d0=seed.get('distances',[0,0])[0],d1=seed.get('distances',[0,0])[1],native_setback_mismatch=c['native_setback_mismatch']))
tasks=proof/'tasks.txt';tasks.write_text(''.join(f"{i} {r['input']} {r['result']} "+' '.join(format(v,'.17g') for p in r['points'] for v in p)+' '+str(r['limit'])+'\n' for i,r in enumerate(rows)))
with (proof/'oracle.txt').open('w') as log:subprocess.run([str(root/'target/cone-contact-oracle'),str(tasks),str(proof/'measurements.json')],env=env,stdout=log,stderr=subprocess.STDOUT,check=True)
measurements=json.loads((proof/'measurements.json').read_text());assert len(measurements)==len(rows)==50
for m in measurements:
 row=rows[m['row']];assert m['valid'];assert len(m['planar_setbacks'])==len(m['cone_setbacks'])==1,(row,m)
 # Output uses planar_* while requests use expected_plane.
 errors=dict(plane=abs(m['planar_setbacks'][0]-row['expected_plane'])+m['planar_spread']/2,cone=abs(m['cone_setbacks'][0]-row['expected_cone'])+m['cone_spread']/2)
 if row['kind']=='native':assert m['planar_circles']>0 and m['cone_circles']>0
 if row['kind']=='native' and row['native_setback_mismatch']:
  assert max(errors.values())>.04
  observed=m['cone_setbacks' if row['reference']=='cone' else 'planar_setbacks'][0]
  folded=row['d0']*row['d1']/(2*row['d0']*math.cos(math.pi/4)-row['d1'])
  assert abs(observed-folded)<1e-10,(row,m,folded)
 else:assert max(errors.values())<1e-6,(row,m,errors)
 row.update(measurement=m,error_bound=errors)
for group in ['kernel_source_sha256','source_sha256','executable_sha256']:
 for p,h in manifest[group].items():assert sha(p)==h,p
(proof/'analysis.json').write_text(json.dumps(dict(kernel_revision=manifest['kernel_revision'],controls=controls,rows=rows,source_artifacts=source_artifacts,oracle_sha256=sha(root/'target/cone-contact-oracle'),source_sha256=sha(proof/'oracle.cpp')),indent=2)+'\n')
print('50 measurements verified: 9 corpus pairs and 16 control pairs; native first-setback fold reproduced',flush=True)
