from pathlib import Path
import json,subprocess,sys
m=json.loads(Path('target/release/.fingerprint/truck-shapeops-cd8bb9657e461910/test-integration-test-tilted_cap_chamfer.json').read_text())
name=sys.argv[1];cmd=['rustc','--edition=2021','--test','--cfg','feature="step-test"','-O','target/tilted_cap_export.rs','-L','dependency=target/release/deps','-o','target/tilted-cap-export']
for _,name,_,value in m['deps']:
 matches=[p for p in Path('target/release/.fingerprint').glob(f'*/lib-{name}') if p.read_text()==value.to_bytes(8,'little').hex()]
 found=[Path('target/release/deps')/f'lib{name}-{p.parent.name.rsplit("-",1)[1]}.rlib' for p in matches]
 found=[p for p in found if p.exists()]
 if len(found)==1:cmd+=['--extern',name+'='+str(found[0])]
subprocess.run(cmd,check=True)
