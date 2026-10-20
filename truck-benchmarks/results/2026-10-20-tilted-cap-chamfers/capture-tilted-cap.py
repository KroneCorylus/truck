from pathlib import Path
import sys,json,subprocess,os,shutil,math
root=Path.cwd();sys.path.insert(0,str(root/'audits/fillet-chamfer'))
from run_occt import isolated_display,execute,tcl_word
out=root/'target/native-tilted-cap';out.mkdir(exist_ok=False)
occt=(root/'../investigations/sequential-holes-2026-10-10/occt').resolve();plugin=root/'target/occt-cone-reference-plugin';sdk=root/'target/occt-sdk/usr/lib64'
lines=['pload ALL','pload -BlendAudit BLENDAUDIT',f'source {tcl_word(root/"audits/fillet-chamfer/capture/blend_capture.tcl")}'];rows=[]
for degrees in [0.,2.,-2.,10.]:
 mode=f'angle{degrees:g}';subprocess.run([str(root/'target/tilted-cap-input'),str(out/f'{mode}.brep'),str(degrees)],check=True)
 i=len(rows);name=f'bundle{i}'
 lines += [f'restore {tcl_word(out/f"{mode}.brep")} {name}',f'explode {name}',f'audit_blend_output before{i} {name}_1',f'catch {{chamf result{i} {name}_1 {name}_2 0.1}} error{i}',f'audit_blend_output after{i} {name}_1'];rows.append(dict(mode=mode,degrees=degrees))
lines.append('exit');script=out/'controls.tcl';script.write_text('\n'.join(lines)+'\n')
with isolated_display(out) as env:
 env.update(LD_LIBRARY_PATH=f'{plugin}:{sdk}',CASROOT=str(occt),CSF_BlendAuditDefaults=str(plugin),CSF_DrawPluginDefaults=str(occt/'src/DrawResources'),DRAWDEFAULT=str(occt/'src/DrawResources/DrawDefault'),CSF_TestScriptsPath=str(occt/'tests'),CSF_ShadersDirectory=str(occt/'src/Shaders'))
 log,code,timeout,elapsed=execute([shutil.which('DRAWEXE'),'-v','-f',str(script)],out,env,120);(out/'controls.log').write_text(log);assert code==0 and not timeout
captures=sorted((out/'capture').glob('*.input.json'));assert len(captures)==len(rows)
for i,(p,q) in enumerate(zip(captures,rows)):
 task=json.loads(p.read_text());result=json.loads(p.with_name(p.name.replace('.input.','.result.')).read_text());assert result['native_valid'] and result['solids']==1,(q,result)
 q['native_input_unchanged']=(out/f'before{i}.result.brep').read_bytes()==(out/f'after{i}.result.brep').read_bytes();assert q['native_input_unchanged'];q.update(prefix=str(p)[:-len('.input.json')],native=result)
 a=1-math.pi/4;c=.4*(10-3*math.pi)/(12-3*math.pi);expected=.782539615254126*(1-.16*a)-math.tan(math.radians(q['degrees']))*(.5-.16*a*(1-c));q.update(expected_input_volume=expected,input_volume_error=task['volume']-expected);assert abs(task['volume']-expected)<1e-10
 print(q['mode'],'input error',q['input_volume_error'],'result volume',result['volume'],'contour lengths',[len(s['contour_edges']) for s in task['selection']],flush=True)
(out/'summary.json').write_text(json.dumps(rows,indent=2)+'\n')
