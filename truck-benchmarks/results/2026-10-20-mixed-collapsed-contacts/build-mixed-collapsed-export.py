from pathlib import Path
import json,subprocess,sys
m=json.loads(Path('target/release/.fingerprint/truck-shapeops-fb6bd51e7e27b261/test-integration-test-R10-sequential-fillet.json').read_text())
name=sys.argv[1];cmd=['rustc','--edition=2021','--test','--cfg','feature="step-test"','-O','target/mixed_collapsed_export.rs','-L','dependency=target/release/deps','-o','target/mixed-collapsed-export']
for _,name,_,value in m['deps']:
 matches=[p for p in Path('target/release/.fingerprint').glob(f'*/lib-{name}') if p.read_text()==value.to_bytes(8,'little').hex()]
 found=[Path('target/release/deps')/f'lib{name}-{p.parent.name.rsplit("-",1)[1]}.rlib' for p in matches]
 found=[p for p in found if p.exists()]
 if len(found)==1:cmd+=['--extern',name+'='+str(found[0])]
subprocess.run(cmd,check=True)
