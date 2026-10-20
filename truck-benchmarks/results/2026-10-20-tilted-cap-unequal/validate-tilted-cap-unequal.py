from pathlib import Path
import subprocess,os,json,hashlib
out=Path('truck-benchmarks/results/2026-10-20-tilted-cap-unequal');previous=Path('truck-benchmarks/results/2026-10-20-tilted-cap-chamfers');old=json.loads((previous/'validation.json').read_text());parent=json.loads((out/'parent.json').read_text());sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest();path='truck-shapeops/examples/replay_occt_blend.rs';marker='#[cfg(test)]\nmod tilted_cap_distance_tests {'
module=marker+Path(path).read_text().split(marker,1)[1];assert hashlib.sha256(module.encode()).hexdigest()==parent['test_module_sha256']
for p,h in old['source_sha256'].items():assert p==path or sha(p)==h,p
sources={p:sha(p) for p in old['source_sha256']};sources.update({str(p):sha(p) for p in Path('truck-shapeops/tests/fixtures/tilted-cap-chamfers').glob('*')})
reused={name:dict(path=str(previous/name),sha256=sha(previous/name)) for name in ['shapeops-tests.txt','stepio-tests.txt','post-cleanup-tests.txt','validation.json','iterator-cleanup.patch']}
m=dict(parent_revision=parent['revision'],source_sha256=sources,reused_unchanged_core=dict(shapeops_tests=442,stepio_tests=115,evidence=reused,iterator_cleanup=old['iterator_cleanup'],other_crates=old['reused_validation']),baseline_failure=old['baseline_failure']);(out/'validation-start.json').write_text(json.dumps(m,indent=2)+'\n')
for name,cmd in [
 ('clippy.txt',['cargo','clippy','--release','-p','truck-shapeops','-p','truck-stepio','-p','truck-benchmarks','--all-targets']),
 ('replay-tests.txt',['cargo','test','--release','-p','truck-shapeops','--example','replay_occt_blend']),
 ('capture-tests.txt',['python3','-m','unittest','discover','-s','audits/fillet-chamfer/capture','-p','test_*.py']),
 ('actions-smoke.txt',['cargo','bench','-p','truck-benchmarks','--bench','actions','--','--test']),
 ('replay-build.txt',['cargo','build','--release','-p','truck-shapeops','--example','replay_occt_blend'])]:
 with (out/name).open('w') as log:
  log.write('$ '+' '.join(cmd)+'\n');log.flush();r=subprocess.run(cmd,env=dict(os.environ,CARGO_BUILD_JOBS='12',**({'RAYON_NUM_THREADS':'1'} if name=='actions-smoke.txt' else {})),stdout=log,stderr=subprocess.STDOUT);log.write(f'exit_code {r.returncode}\n')
 assert r.returncode==0,name;print(name,'passed',flush=True)
for p,h in sources.items():assert sha(p)==h,p
assert '62 passed; 0 failed' in (out/'replay-tests.txt').read_text();m.update(replay_tests=62,replay_sha256=sha('target/release/examples/replay_occt_blend'))
for p in out.glob('*.txt'):p.write_text('\n'.join(line.rstrip() for line in p.read_text().splitlines()).rstrip()+'\n')
m['log_sha256']={p.name:sha(p) for p in out.glob('*.txt')};(out/'validation.json').write_text(json.dumps(m,indent=2)+'\n');print('Validation complete',flush=True)
