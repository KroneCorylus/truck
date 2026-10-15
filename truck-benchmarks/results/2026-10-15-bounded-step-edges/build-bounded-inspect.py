from pathlib import Path
import json, subprocess
root=Path('target/release/.fingerprint')
f=json.loads((root/'truck-shapeops-95c9a5594d96e759/example-replay_occt_blend.json').read_text())
found={}
def visit(f):
 for _,name,_,h in f['deps']:
  if name in found: continue
  for p in root.glob('*/lib-'+name):
   if int.from_bytes(bytes.fromhex(p.read_text()),'little')==h:
    found[name]=p.parent.name.rsplit('-',1)[1]
    visit(json.loads(p.with_suffix('.json').read_text()))
    break
visit(f)
for source,names in [('inspect-public-chamfer',['serde_json','truck_meshalgo','truck_modeling','truck_shapeops','truck_stepio']),('inspect-bounded-residuals',['ruststep','truck_stepio'])]:
 cmd=['rustc','--edition=2021','-Awarnings','target/'+source+'.rs','-L','dependency=target/release/deps','-o','target/'+source+'-fixed']
 for name in names:cmd+=['--extern',f'{name}=target/release/deps/lib{name}-{found[name]}.rlib']
 subprocess.run(cmd,check=True)
