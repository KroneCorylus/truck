from pathlib import Path
import sys,json,subprocess,os,shutil,math
root=Path.cwd();sys.path.insert(0,str(root/'audits/fillet-chamfer'))
from run_occt import isolated_display,execute,tcl_word
out=(root/sys.argv[2]).resolve();out.mkdir(exist_ok=False)
occt=(root/'../investigations/sequential-holes-2026-10-10/occt').resolve();plugin=(root/sys.argv[1]).resolve();sdk=root/'target/occt-sdk/usr/lib64'
with (out/'inputs.log').open('w') as log:subprocess.run([str(root/'target/cone-reference-native-inputs'),str(out)],env=dict(os.environ,LD_LIBRARY_PATH=str(sdk)),stdout=log,stderr=subprocess.STDOUT,check=True)
lines=['pload ALL','pload -BlendAudit BLENDAUDIT',f'source {tcl_word(root/"audits/fillet-chamfer/capture/blend_capture.tcl")}']
controls=[]
for k in range(2):
 for ref in [3,4]:
  for mode in ['unequal','angle']:
   i=len(controls);name=f'bundle{i}';request='0.3 0.7' if mode=='unequal' else 'A 0.3 30'
   lines += [f'restore {tcl_word(out/f"input-{k}.brep")} {name}',f'explode {name}',f'audit_blend_output before{i} {name}_1',f'chamf result{i} {name}_1 {name}_2 {name}_{ref} {request}',f'audit_blend_output after{i} {name}_1']
   controls.append(dict(input=k,reference='plane' if ref==3 else 'cone',mode=mode))
lines.append('exit');script=out/'controls.tcl';script.write_text('\n'.join(lines)+'\n')
with isolated_display(out) as env:
 env.update(LD_LIBRARY_PATH=f'{plugin}:{sdk}',CASROOT=str(occt),CSF_BlendAuditDefaults=str(plugin),CSF_DrawPluginDefaults=str(occt/'src/DrawResources'),DRAWDEFAULT=str(occt/'src/DrawResources/DrawDefault'),CSF_TestScriptsPath=str(occt/'tests'),CSF_ShadersDirectory=str(occt/'src/Shaders'))
 log,code,timeout,elapsed=execute([shutil.which('DRAWEXE'),'-v','-f',str(script)],out,env,120);(out/'controls.log').write_text(log);assert code==0 and not timeout
captures=sorted((out/'capture').glob('*.input.json'));assert len(captures)==8
for i,(p,c) in enumerate(zip(captures,controls)):
 result=json.loads(p.with_name(p.name.replace('.input.','.result.')).read_text());assert result['native_valid'] and result['solids']==1
 slope=-.2 if c['input']==0 else .2;top=3 if c['input']==0 else 5;g=math.sqrt(1+slope*slope)
 # Interior angle between the cap's inward radial direction and the cone generator.
 cosine=slope/g;sine=1/g
 d0=.3;d1=.7 if c['mode']=='unequal' else d0*math.tan(math.pi/6)/(sine+cosine*math.tan(math.pi/6))
 radial,along=(d0,d1) if c['reference']=='plane' else (d1,d0)
 vertical=along/g;expected=math.pi*10*49/3-math.pi*(top*radial*vertical-slope*radial*vertical*vertical/3-radial*radial*vertical/3)
 c.update(prefix=str(p)[:-len('.input.json')],expected_volume=expected,native_volume=result['volume'],radial=radial,along=along,vertical=vertical)
 assert abs(expected-result['volume'])<1e-7,(c,result)
 assert (out/f'before{i}.result.brep').read_bytes()==(out/f'after{i}.result.brep').read_bytes()
 print(c,flush=True)
(out/'summary.json').write_text(json.dumps(controls,indent=2)+'\n')

for p,c in zip(captures,controls):
 task=json.loads(p.read_text());selection=task['selection'][0]
 assert task['schema_version']==2 and len(selection['contour_edges'])==1
 if c['reference']=='cone':
  reference=selection.get('reference_cone')
  assert reference is not None, 'Missing reference_cone for '+str(p)
  assert selection['reference_plane'] is None
  assert reference['apex']==[0,0,25 if c['input']==0 else -15]
  assert reference['axis']==[0,0,-1 if c['input']==0 else 1]
  assert abs(reference['half_angle_radians']-math.atan(.2))<1e-12
 else:
  assert selection['reference_plane'] is not None and 'reference_cone' not in selection
print('Native cone descriptors, both reference sides, slopes, parameter modes and input immutability verified')
