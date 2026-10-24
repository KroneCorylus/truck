import hashlib, json, subprocess
from pathlib import Path
root=Path.cwd()
backup=root/'target/d-basis-parent-source'
backup.mkdir(exist_ok=False)
changed=subprocess.check_output(['git','diff','--name-only'],text=True).splitlines()
paths=[p for p in changed if '/src/' in p and p.endswith('.rs')]
original={p:Path(p).read_bytes() for p in paths}
for p,data in original.items():
    dest=backup/p
    dest.parent.mkdir(parents=True,exist_ok=True)
    dest.write_bytes(data)
(backup/'source.json').write_text(json.dumps({p:hashlib.sha256(data).hexdigest() for p,data in original.items()},indent=2)+'\n')
try:
    for p in paths:
        Path(p).write_bytes(subprocess.check_output(['git','show','HEAD:'+p]))
    result=subprocess.run(['python3','target/run-fillet-validation-2026-10-24.py','truck-benchmarks/results/2026-10-24-exact-knots/parent-basis-contract.txt','cargo','test','--release','-p','truck-geometry','--test','basis_arithmetic_contract'])
finally:
    for p,data in original.items():
        Path(p).write_bytes(data)
assert result.returncode == 101, result.returncode
