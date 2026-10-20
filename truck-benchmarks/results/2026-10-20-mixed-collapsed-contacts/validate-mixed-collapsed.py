from pathlib import Path
import subprocess,os,json,hashlib,re
out=Path('truck-benchmarks/results/2026-10-20-mixed-collapsed-contacts');env=dict(os.environ,CARGO_BUILD_JOBS='12');sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest();parent=json.loads((out/'parent.json').read_text())
assert sha('truck-shapeops/tests/mixed_collapsed_contacts.rs')==parent['sha256']['truck-shapeops/tests/mixed_collapsed_contacts.rs'];assert '1 passed; 2 failed' in (out/'parent-tests.txt').read_text()
previous=Path('truck-benchmarks/results/2026-10-20-procedural-cylinders');prior=json.loads((previous/'validation.json').read_text());reused={}
for crate,name in [('truck-geometry','geometry'),('truck-modeling','modeling'),('truck-meshalgo','meshalgo')]:
 sources={p:h for p,h in prior['source_sha256'].items() if p.startswith(crate+'/')};assert sources
 for p,h in sources.items():assert sha(p)==h,p
 log=previous/f'{name}-tests.txt';assert sha(log)==prior['log_sha256'][log.name]
 reused[name]=dict(counts=prior[name+'_counts'],source_sha256=sources,log=str(log),log_sha256=sha(log))
source={str(p):sha(p) for base in ['truck-shapeops/src','truck-shapeops/tests','truck-shapeops/examples','truck-meshalgo/src','truck-meshalgo/tests','truck-geometry/src','truck-geometry/tests','truck-modeling/src','truck-modeling/tests','truck-stepio/src','truck-stepio/tests'] for p in Path(base).rglob('*.rs')}
m=dict(parent_revision=parent['revision'],source_sha256=source,reused_validation=reused,baseline_failure=prior['baseline_failure']);(out/'validation-start.json').write_text(json.dumps(m,indent=2)+'\n')
for name,cmd in [
 ('shapeops-tests.txt',['cargo','test','--release','-p','truck-shapeops']),
 ('stepio-tests.txt',['cargo','test','--release','-p','truck-stepio']),
 ('replay-tests.txt',['cargo','test','--release','-p','truck-shapeops','--example','replay_occt_blend']),
 ('capture-tests.txt',['python3','-m','unittest','discover','-s','audits/fillet-chamfer/capture','-p','test_*.py']),
 ('clippy.txt',['cargo','clippy','--release','-p','truck-shapeops','-p','truck-stepio','-p','truck-benchmarks','--all-targets']),
 ('actions-smoke.txt',['cargo','bench','-p','truck-benchmarks','--bench','actions','--','--test']),
 ('replay-build.txt',['cargo','build','--release','-p','truck-shapeops','--example','replay_occt_blend'])]:
 with (out/name).open('w') as log:
  log.write('$ '+' '.join(cmd)+'\n');log.flush();r=subprocess.run(cmd,env=dict(env,RAYON_NUM_THREADS='1') if name=='actions-smoke.txt' else env,stdout=log,stderr=subprocess.STDOUT);log.write(f'exit_code {r.returncode}\n')
 assert r.returncode==0,name;print(name,'passed',flush=True)
for p,h in source.items():assert sha(p)==h,p
for name,expected in [('shapeops',[438,0,1]),('stepio',[115,0,0])]:
 counts=[tuple(map(int,c)) for c in re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored',(out/f'{name}-tests.txt').read_text())];m[name+'_counts']=list(map(sum,zip(*counts)));assert m[name+'_counts']==expected,m[name+'_counts']
m.update(replay_tests=59,replay_sha256=sha('target/release/examples/replay_occt_blend'));assert '59 passed; 0 failed' in (out/'replay-tests.txt').read_text()
for p in out.glob('*.txt'):p.write_text('\n'.join(line.rstrip() for line in p.read_text().splitlines()).rstrip()+'\n')
m['log_sha256']={p.name:sha(p) for p in out.glob('*.txt')};(out/'validation.json').write_text(json.dumps(m,indent=2)+'\n');print('Validation complete',flush=True)
