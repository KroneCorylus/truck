from pathlib import Path
import json,hashlib,subprocess
out=Path('truck-benchmarks/results/2026-10-20-procedural-cylinders');sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest();validation=json.loads((out/'validation.json').read_text())
for p,h in validation['source_sha256'].items():assert sha(p)==h,p
assert sha('target/release/examples/replay_occt_blend')==validation['replay_sha256']
references={str(p):sha(p) for p in Path('target/native-procedural-cylinders').rglob('*') if p.is_file()}
subprocess.run(['python3','target/build-procedural-cylinders-export.py','final'],check=True)
subprocess.run(['target/procedural-cylinders-export','export_procedural_controls','--exact'],check=True)
subprocess.run(['python3','target/compare-procedural-cylinders.py'],check=True)
for p,h in references.items():assert sha(p)==h,p
for p,h in validation['source_sha256'].items():assert sha(p)==h,p
assert sha('target/release/examples/replay_occt_blend')==validation['replay_sha256']
records=json.loads(Path('target/procedural-cylinders-comparison-summary.json').read_text());assert len(records)==4
Path('target/procedural-cylinders-validated-summary.json').write_text(json.dumps(dict(controls=records,native_reference_sha256=references,producer_sha256=sha('target/procedural-cylinders-export'),producer_source_sha256=sha('target/procedural_cylinders_export.rs')),indent=2)+'\n')
