from pathlib import Path
import subprocess,os,json,hashlib,re,difflib
out=Path('truck-benchmarks/results/2026-10-20-tilted-cap-chamfers');env=dict(os.environ,CARGO_BUILD_JOBS='12');sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest();m=json.loads((out/'validation-start.json').read_text());old=m['source_sha256'];path='truck-shapeops/src/fillet/chamfer.rs';current=Path(path).read_text()
a='''            for (source, edge) in support_contacts[k].iter().zip(&trimmed.contacts[k]) {
                let curve = constrain_contact(source, edge, tol)'''
b='''            for side in 0..2 {
                let edge = &trimmed.contacts[k][side];
                let curve = constrain_contact(&support_contacts[k][side], edge, tol)'''
assert current.count(a)==1
prior=current.replace(a,b);assert hashlib.sha256(prior.encode()).hexdigest()==old[path]
for p,h in old.items():assert p==path or sha(p)==h,p
(out/'iterator-cleanup.patch').write_text(''.join(difflib.unified_diff(prior.splitlines(True),current.splitlines(True),fromfile=path+' (full suite)',tofile=path+' (final)')))
m['full_suite_source_sha256']=old;m['source_sha256']={p:sha(p) for p in old};m['iterator_cleanup']='Only the indexed two-element contact loop was rewritten as an iterator zip, preserving order and both arrays. Full suites passed before cleanup; seven affected integration targets and replay tests passed afterward. The exact patch and both source hashes are retained.'
for name,expected in [('shapeops',[442,0,1]),('stepio',[115,0,0])]:
 counts=[tuple(map(int,c)) for c in re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored',(out/f'{name}-tests.txt').read_text())];m[name+'_counts']=list(map(sum,zip(*counts)));assert m[name+'_counts']==expected,m[name+'_counts']
assert '59 passed; 0 failed' in (out/'replay-tests.txt').read_text()
(out/'clippy-before-cleanup.txt').write_bytes((out/'clippy.txt').read_bytes())
for name,cmd in [
 ('clippy.txt',['cargo','clippy','--release','-p','truck-shapeops','-p','truck-stepio','-p','truck-benchmarks','--all-targets']),
 ('post-cleanup-tests.txt',['cargo','test','--release','-p','truck-shapeops',*[x for t in ['tilted_cap_chamfer','chamfer_general','disjoint_chamfer_rims','chamfer_terminal_growth','conic_section_chamfers','mixed_chamfer_components','tangent_chamfer_fit'] for x in ['--test',t]]]),
 ('post-cleanup-replay-tests.txt',['cargo','test','--release','-p','truck-shapeops','--example','replay_occt_blend']),
 ('actions-smoke.txt',['cargo','bench','-p','truck-benchmarks','--bench','actions','--','--test']),
 ('replay-build.txt',['cargo','build','--release','-p','truck-shapeops','--example','replay_occt_blend'])]:
 with (out/name).open('w') as log:
  log.write('$ '+' '.join(cmd)+'\n');log.flush();r=subprocess.run(cmd,env=dict(env,RAYON_NUM_THREADS='1') if name=='actions-smoke.txt' else env,stdout=log,stderr=subprocess.STDOUT);log.write(f'exit_code {r.returncode}\n')
 assert r.returncode==0,name;print(name,'passed',flush=True)
for p,h in m['source_sha256'].items():assert sha(p)==h,p
m.update(replay_tests=59,replay_sha256=sha('target/release/examples/replay_occt_blend'));assert '59 passed; 0 failed' in (out/'post-cleanup-replay-tests.txt').read_text()
for p in out.glob('*.txt'):p.write_text('\n'.join(line.rstrip() for line in p.read_text().splitlines()).rstrip()+'\n')
m['log_sha256']={p.name:sha(p) for p in out.glob('*.txt')};(out/'validation.json').write_text(json.dumps(m,indent=2)+'\n');print('Validation complete',flush=True)
