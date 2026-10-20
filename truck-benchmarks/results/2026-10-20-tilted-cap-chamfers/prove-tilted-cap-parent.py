from pathlib import Path
import subprocess,os,json,hashlib,shutil
out=Path('truck-benchmarks/results/2026-10-20-tilted-cap-chamfers');paths=['truck-shapeops/src/fillet/chamfer.rs','truck-shapeops/src/fillet/modeling.rs'];saved={p:Path(p).read_bytes() for p in paths};shutil.copyfile(out/'parent-tests.txt',out/'parent-before-setback-tests.txt')
try:
 for p in paths:Path(p).write_bytes(subprocess.check_output(['git','show','HEAD:'+p]))
 with (out/'parent-tests.txt').open('w') as log:
  r=subprocess.run(['cargo','test','--release','-p','truck-shapeops','--test','tilted_cap_chamfer'],env=dict(os.environ,CARGO_BUILD_JOBS='12'),stdout=log,stderr=subprocess.STDOUT);log.write(f'exit_code {r.returncode}\n')
 assert r.returncode==101 and '2 passed; 2 failed' in (out/'parent-tests.txt').read_text()
 m=json.loads((out/'parent.json').read_text());m['sha256']['truck-shapeops/tests/tilted_cap_chamfer.rs']=hashlib.sha256(Path('truck-shapeops/tests/tilted_cap_chamfer.rs').read_bytes()).hexdigest();m['tests']=dict(passed=2,failed=2);(out/'parent.json').write_text(json.dumps(m,indent=2)+'\n')
finally:
 for p,data in saved.items():Path(p).write_bytes(data)
print('Parent proof complete; candidate restored')
