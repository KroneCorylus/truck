from pathlib import Path
import sys,json,subprocess,os,shutil,hashlib
root=Path.cwd();sys.path.insert(0,str(root/'audits/fillet-chamfer'))
from run_occt import isolated_display,execute,tcl_word
out=root/'target/native-reordered-loops';assert not (out/'capture').exists()
occt=(root/'../investigations/sequential-holes-2026-10-10/occt').resolve();plugin=root/'target/occt-cone-reference-plugin';sdk=root/'target/occt-sdk/usr/lib64'
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
lines=['pload ALL','pload -BlendAudit BLENDAUDIT',f'source {tcl_word(root/"audits/fillet-chamfer/capture/blend_capture.tcl")}'];controls=[]
for shape in ['outer','inner']:
 for mode,axial in [('equal',.2),('unequal',.1)]:
  i=len(controls);name=f'bundle{i}'
  request=f'{name}_2 .2' if mode=='equal' else (f'{name}_2 {name}_3 A 1 30' if mode=='angle' else f'{name}_2 {name}_3 .2 {axial}')
  lines += [f'restore {tcl_word(out/(shape+".brep"))} {name}',f'explode {name}',f'audit_blend_output before{i} {name}_1',f'catch {{chamf result{i} {name}_1 {request}}} error{i}',f'audit_blend_output after{i} {name}_1']
  controls.append(dict(shape=shape,mode=mode,planar=.2,axial=axial,length=2. if shape=='inner' else 10.))
lines.append('exit');script=out/'controls.tcl';script.write_text('\n'.join(lines)+'\n')
with isolated_display(out) as env:
 env.update(LD_LIBRARY_PATH=f'{plugin}:{sdk}',CASROOT=str(occt),CSF_BlendAuditDefaults=str(plugin),CSF_DrawPluginDefaults=str(occt/'src/DrawResources'),DRAWDEFAULT=str(occt/'src/DrawResources/DrawDefault'),CSF_TestScriptsPath=str(occt/'tests'),CSF_ShadersDirectory=str(occt/'src/Shaders'))
 log,code,timeout,elapsed=execute([shutil.which('DRAWEXE'),'-v','-f',str(script)],out,env,120);(out/'controls.log').write_text(log);assert code==0 and not timeout
captures=sorted((out/'capture').glob('*.input.json'));assert len(captures)==len(controls)
for i,(p,c) in enumerate(zip(captures,controls)):
 result=json.loads(p.with_name(p.name.replace('.input.','.result.')).read_text());assert result['native_valid'] and result['solids']==1,(c,result)
 c['native_input_unchanged']=(out/f'before{i}.result.brep').read_bytes()==(out/f'after{i}.result.brep').read_bytes()
 c.update(prefix=str(p)[:-len('.input.json')],native=result)
 expected=460-c['length']*c['planar']*c['axial']/2
 c.update(expected_volume=expected,native_volume_error=result['volume']-expected)
 assert abs(result['volume']-expected)<1e-7,(c,result)
(out/'summary.json').write_text(json.dumps(controls,indent=2)+'\n')
print(json.dumps(controls,indent=2))
