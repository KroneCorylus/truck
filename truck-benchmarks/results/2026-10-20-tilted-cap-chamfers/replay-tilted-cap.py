from pathlib import Path
import json,hashlib,subprocess
out=Path('truck-benchmarks/results/2026-10-20-tilted-cap-chamfers');sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest();validation=json.loads((out/'validation.json').read_text())
for p,h in validation['source_sha256'].items():assert sha(p)==h,p
assert sha('target/release/examples/replay_occt_blend')==validation['replay_sha256']
references={str(p):sha(p) for p in Path('target/native-tilted-cap').rglob('*') if p.is_file()}
subprocess.run(['python3','target/build-tilted-cap-export.py','final'],check=True)
subprocess.run(['target/tilted-cap-export','export_tilted_controls','--exact'],check=True)
with open('target/tilted-cap-final-native-comparison.txt','w') as log:r=subprocess.run(['python3','target/compare-tilted-cap.py'],stdout=log,stderr=subprocess.STDOUT)
assert r.returncode==1
assert "[('angle10', 'result', 'distance')]" in Path('target/tilted-cap-final-native-comparison.txt').read_text()
subprocess.run(['python3','target/prove-tilted-cap-discrepancy.py'],check=True)
for p,h in references.items():assert sha(p)==h,p
for p,h in validation['source_sha256'].items():assert sha(p)==h,p
assert sha('target/release/examples/replay_occt_blend')==validation['replay_sha256']
records=json.loads(Path('target/tilted-cap-comparison-summary.json').read_text());assert len(records)==4
Path('target/tilted-cap-validated-summary.json').write_text(json.dumps(dict(controls=records,native_reference_sha256=references,producer_sha256=sha('target/tilted-cap-export'),producer_source_sha256=sha('target/tilted_cap_export.rs'),native_comparison=dict(passed=3,failed=1,failed_control='angle10',failed_check='distance',threshold=.001),discrepancy_proof=json.load(open('target/tilted-cap-discrepancy-proof.json'))),indent=2)+'\n')
