from pathlib import Path
import json,subprocess,hashlib
out=Path('truck-benchmarks/results/2026-10-20-collapsed-fillet-contacts');sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest();validation=json.loads((out/'validation.json').read_text())
for p,h in validation['source_sha256'].items():assert sha(p)==h,p
assert sha('target/release/examples/replay_occt_blend')==validation['replay_sha256']
inputs={}
for folder in ['native-collapsed-contacts','native-collapsed-contacts-closed']:
 base=Path('target')/folder
 for p in base.rglob('*'):
  if p.is_file() and ('.input.' in p.name or '.result.' in p.name):inputs[str(p)]=sha(p)
 for q in json.loads((base/'summary.json').read_text()):assert q['native_input_unchanged'] and q['native']['native_valid'] and q['native']['solids']==1
subprocess.run(['python3','target/replay-collapsed-contacts-probe.py'],check=True)
for p,h in inputs.items():assert sha(p)==h,p
for p,h in validation['source_sha256'].items():assert sha(p)==h,p
records=json.loads(Path('target/collapsed-contacts-replay-summary.json').read_text());assert len(records)==4
Path('target/collapsed-contacts-validated-summary.json').write_text(json.dumps(dict(controls=records,input_reference_sha256=inputs),indent=2)+'\n')
