from pathlib import Path
import subprocess,os,json,hashlib,shutil
out=Path('truck-benchmarks/results/2026-10-20-tilted-cap-unequal');p=Path('truck-shapeops/examples/replay_occt_blend.rs');saved=p.read_text();marker='#[cfg(test)]\nmod tilted_cap_distance_tests {';base=subprocess.check_output(['git','show','HEAD:'+str(p)],text=True);tests=saved[saved.index(marker):];parent=base+'\n'+tests;shutil.copyfile(out/'parent-tests.txt',out/'parent-before-contact-tests.txt')
try:
 p.write_text(parent)
 with (out/'parent-tests.txt').open('w') as log:r=subprocess.run(['cargo','test','--release','-p','truck-shapeops','--example','replay_occt_blend','tilted_cap_distance_tests'],env=dict(os.environ,CARGO_BUILD_JOBS='12'),stdout=log,stderr=subprocess.STDOUT);log.write(f'exit_code {r.returncode}\n')
 assert r.returncode==101 and '2 passed; 1 failed' in (out/'parent-tests.txt').read_text()
 m=json.loads((out/'parent.json').read_text());m['source_sha256'][str(p)]=hashlib.sha256(parent.encode()).hexdigest();m['test_module_sha256']=hashlib.sha256(tests.encode()).hexdigest();(out/'parent.json').write_text(json.dumps(m,indent=2)+'\n')
finally:p.write_text(saved)
print('Strengthened parent proof complete; candidate restored')
