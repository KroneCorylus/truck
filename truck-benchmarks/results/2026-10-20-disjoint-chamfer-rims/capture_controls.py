from pathlib import Path
import sys,json,subprocess,os,shutil,hashlib
root=Path.cwd();sys.path.insert(0,str(root/'audits/fillet-chamfer'))
from run_occt import isolated_display,execute,tcl_word
out=root/'target/native-disjoint-rims';assert not (out/'capture').exists()
occt=(root/'../investigations/sequential-holes-2026-10-10/occt').resolve();plugin=root/'target/occt-cone-reference-plugin';sdk=root/'target/occt-sdk/usr/lib64'
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
lines=['pload ALL','pload -BlendAudit BLENDAUDIT',f'source {tcl_word(root/"audits/fillet-chamfer/capture/blend_capture.tcl")}'];controls=[]
for shape in ['quarter','full']:
 for mode in ['equal','unequal','angle','different']:
  i=len(controls);name=f'bundle{i}';pairs=[[.7,.7],[.7,.7]] if mode=='equal' else ([[.7,.5],[.4,.3]] if mode=='different' else [[.7,.5],[.7,.5]])
  if mode=='angle':pairs=[[.7,.7/(3**.5)]]*2
  request=' '.join(f'{name}_{j+2} {d[0]}' if mode=='equal' else (f'{name}_{j+2} {name}_{j+4} A {d[0]} 30' if mode=='angle' else f'{name}_{j+2} {name}_{j+4} {d[0]} {d[1]}') for j,d in enumerate(pairs))
  lines += [f'restore {tcl_word(out/(shape+".brep"))} {name}',f'explode {name}',f'audit_blend_output before{i} {name}_1',f'catch {{chamf result{i} {name}_1 {request}}} error{i}',f'audit_blend_output after{i} {name}_1']
  controls.append(dict(shape=shape,mode=mode,distances=pairs,radius=5.,height=10.))
lines.append('exit');script=out/'controls.tcl';script.write_text('\n'.join(lines)+'\n')
with isolated_display(out) as env:
 env.update(LD_LIBRARY_PATH=f'{plugin}:{sdk}',CASROOT=str(occt),CSF_BlendAuditDefaults=str(plugin),CSF_DrawPluginDefaults=str(occt/'src/DrawResources'),DRAWDEFAULT=str(occt/'src/DrawResources/DrawDefault'),CSF_TestScriptsPath=str(occt/'tests'),CSF_ShadersDirectory=str(occt/'src/Shaders'))
 log,code,timeout,elapsed=execute([shutil.which('DRAWEXE'),'-v','-f',str(script)],out,env,120);(out/'controls.log').write_text(log);assert code==0 and not timeout
captures=sorted((out/'capture').glob('*.input.json'));assert len(captures)==len(controls)
for i,(p,c) in enumerate(zip(captures,controls)):
 result=json.loads(p.with_name(p.name.replace('.input.','.result.')).read_text());assert result['native_valid'] and result['solids']==1,(c,result)
 c['native_input_unchanged']=(out/f'before{i}.result.brep').read_bytes()==(out/f'after{i}.result.brep').read_bytes()
 c.update(prefix=str(p)[:-len('.input.json')],native=result)
 theta=2*__import__('math').pi if c['shape']=='full' else __import__('math').pi/2
 expected=theta/2*c['radius']**2*c['height']-sum(theta/2*axial*(c['radius']*radial-radial**2/3) for radial,axial in c['distances'])
 c.update(expected_volume=expected,native_volume_error=result['volume']-expected)
 assert abs(result['volume']-expected)<1e-7,(c,result)
(out/'summary.json').write_text(json.dumps(controls,indent=2)+'\n')
print(json.dumps(controls,indent=2))
