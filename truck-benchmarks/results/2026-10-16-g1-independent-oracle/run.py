from pathlib import Path
import hashlib,json,os,subprocess,zipfile

root=Path.cwd();out=root/'truck-benchmarks/results/2026-10-16-g1-independent-oracle'
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
baseline=root/'audits/fillet-chamfer/capture/2026-10-15-native-tolerances'
m=json.loads((baseline/'results.json').read_text())
assert sha(baseline/'artifacts.zip')==m['artifact_archive_sha256']
case=next(c for c in m['cases'] if c['id']=='blend/simple/G1')
raw=root/'target/occt-capture-native-tolerances'
inputs={p:sha(raw/p) for p in case['artifacts']}
with zipfile.ZipFile(baseline/'artifacts.zip') as z:
 for p,h in inputs.items():assert hashlib.sha256(z.read(p)).hexdigest()==h==case['artifacts'][p],p
q=case['captures'][0]
assert q['task']['selection'][0]['radius']==1000
assert q['task']['contours'][0]['constant_radius']==1000
assert q['task']['native_settings']['representation']=='rational'
source=out/'check_g1.cpp';source_hash=sha(source);binary=root/'target/check-g1-oracle'
cmd=['g++','-std=c++17','-O2','-Wall','-Wextra','-Wpedantic','-isystem','target/occt-sdk/usr/include/opencascade',str(source),'-Ltarget/occt-sdk/usr/lib64','-Wl,-rpath-link,target/occt-sdk/usr/lib64','-lTKDESTEP','-lTKXSBase','-lTKFillet','-lTKTopAlgo','-lTKGeomAlgo','-lTKBRep','-lTKGeomBase','-lTKG3d','-lTKG2d','-lTKMath','-lTKernel','-o',str(binary)]
with (out/'build.txt').open('w') as log:
 log.write('$ '+' '.join(cmd)+'\n');log.flush();subprocess.run(cmd,stdout=log,stderr=subprocess.STDOUT,check=True);log.write('exit_code 0\n')
env=dict(os.environ,LD_LIBRARY_PATH=str(root/'target/occt-sdk/usr/lib64'))
prefix=raw/'blend/simple/G1/capture/001'
summary={}
for grid in [32,64,128]:
 cmd=[str(binary),str(prefix)+'.input.brep',str(prefix)+'.result.brep',str(prefix)+'.truck.step',str(grid)]
 with (out/f'grid-{grid}.txt').open('w') as log:
  log.write('$ '+' '.join(cmd)+'\n');log.flush();subprocess.run(cmd,stdout=log,stderr=subprocess.STDOUT,env=env,check=True,timeout=180);log.write('exit_code 0\n')
 data={}
 for line in (out/f'grid-{grid}.txt').read_text().splitlines():
  parts=line.split()
  if parts and parts[0] in ['native','Truck','native-default','native-tight']:
   data[parts[0]]={k:float(v) if ',' not in v else [float(x) for x in v.split(',')] for k,v in (p.split('=') for p in parts[1:])}
 assert set(data)=={'native','Truck','native-default','native-tight'}
 assert data['native']['sampled_radius_residual']>.1
 assert data['native']['plane_contact_distance']>.3
 assert data['Truck']['sampled_radius_residual']<.001
 assert data['Truck']['plane_contact_distance']<.001
 for label in ['native-default','native-tight']:
  for field in ['sampled_radius_residual','plane_contact_distance']:
   assert abs(data[label][field]-data['native'][field])<1e-6,(label,field)
 summary[str(grid)]=data
 print('Validated grid',grid,flush=True)
ldd=subprocess.check_output(['ldd',str(binary)],env=env,text=True)
(out/'libraries.txt').write_text(ldd)
libs={}
for line in ldd.splitlines():
 parts=line.split()
 if len(parts)>=3 and parts[1]=='=>' and Path(parts[2]).is_file():libs[parts[2]]=sha(parts[2])
assert sha(source)==source_hash
for p,h in inputs.items():assert sha(raw/p)==h,p
metadata=dict(parent_revision=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),baseline_revision=m['kernel_revision'],baseline_manifest_sha256=sha(baseline/'results.json'),baseline_archive_sha256=m['artifact_archive_sha256'],source_sha256={str(source.relative_to(root)):source_hash},executable_sha256=sha(binary),libraries_sha256=libs,input_sha256=inputs,settings=q['task']['native_settings'],summary=summary)
(out/'metadata.json').write_text(json.dumps(metadata,indent=2)+'\n')
print('All analytic, shifted-radius, domain, input immutability and sampling controls passed')
