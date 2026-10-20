from pathlib import Path
import json,hashlib,subprocess
out=Path('truck-benchmarks/results/2026-10-20-mixed-collapsed-contacts');sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest();validation=json.loads((out/'validation.json').read_text())
for p,h in validation['source_sha256'].items():assert sha(p)==h,p
assert sha('target/release/examples/replay_occt_blend')==validation['replay_sha256']
references={str(p):sha(p) for p in Path('target/native-mixed-collapsed-rims').rglob('*') if p.is_file()}
subprocess.run(['python3','target/build-mixed-collapsed-export.py','final'],check=True)
subprocess.run(['target/mixed-collapsed-export','export_mixed_controls','--exact'],check=True)
subprocess.run(['python3','target/compare-mixed-collapsed.py'],check=True)
for p,h in references.items():assert sha(p)==h,p
for p,h in validation['source_sha256'].items():assert sha(p)==h,p
assert sha('target/release/examples/replay_occt_blend')==validation['replay_sha256']
records=json.loads(Path('target/mixed-collapsed-comparison-summary.json').read_text());assert len(records)==2
Path('target/mixed-collapsed-validated-summary.json').write_text(json.dumps(dict(controls=records,native_reference_sha256=references,producer_sha256=sha('target/mixed-collapsed-export'),producer_source_sha256=sha('target/mixed_collapsed_export.rs')),indent=2)+'\n')
