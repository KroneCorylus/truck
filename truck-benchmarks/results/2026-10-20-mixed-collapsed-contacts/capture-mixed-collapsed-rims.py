from pathlib import Path
import sys,json,subprocess,os,shutil,math
root=Path.cwd();sys.path.insert(0,str(root/'audits/fillet-chamfer'))
from run_occt import isolated_display,execute,tcl_word
out=root/'target/native-mixed-collapsed-rims';out.mkdir(exist_ok=True);assert not (out/'capture').exists()
occt=(root/'../investigations/sequential-holes-2026-10-10/occt').resolve();plugin=root/'target/occt-cone-reference-plugin';sdk=root/'target/occt-sdk/usr/lib64'
lines=['pload ALL','pload -BlendAudit BLENDAUDIT',f'source {tcl_word(root/"audits/fillet-chamfer/capture/blend_capture.tcl")}'];rows=[]
for size,radius in [(12.,0.5),(12.,1.)]:
 mode=f'size{size:g}-radius{radius:g}'
 subprocess.run([str(root/'target/mixed-collapsed-input'),str(out/f'{mode}.brep')],check=True)
 i=len(rows);name=f'bundle{i}'
 lines += [f'restore {tcl_word(out/f"{mode}.brep")} {name}',f'explode {name}',f'audit_blend_output before{i} {name}_1',f'catch {{blend result{i} {name}_1 {radius} {name}_2}} error{i}',f'audit_blend_output after{i} {name}_1']
 rows.append(dict(mode=mode,size=size,radius=radius))
lines.append('exit');script=out/'controls.tcl';script.write_text('\n'.join(lines)+'\n')
with isolated_display(out) as env:
 env.update(LD_LIBRARY_PATH=f'{plugin}:{sdk}',CASROOT=str(occt),CSF_BlendAuditDefaults=str(plugin),CSF_DrawPluginDefaults=str(occt/'src/DrawResources'),DRAWDEFAULT=str(occt/'src/DrawResources/DrawDefault'),CSF_TestScriptsPath=str(occt/'tests'),CSF_ShadersDirectory=str(occt/'src/Shaders'))
 log,code,timeout,elapsed=execute([shutil.which('DRAWEXE'),'-v','-f',str(script)],out,env,120);(out/'controls.log').write_text(log);assert code==0 and not timeout
captures=sorted((out/'capture').glob('*.input.json'));assert len(captures)==len(rows)
for i,(p,q) in enumerate(zip(captures,rows)):
 result=json.loads(p.with_name(p.name.replace('.input.','.result.')).read_text());assert result['native_valid'] and result['solids']==1,(q,result)
 q['native_input_unchanged']=(out/f'before{i}.result.brep').read_bytes()==(out/f'after{i}.result.brep').read_bytes();q.update(prefix=str(p)[:-len('.input.json')],native=result)
 r,size=q['radius'],q['size'];area=r*r*(1-math.pi/4);centroid=r*(10-3*math.pi)/(12-3*math.pi);length=2*(12+10)-2*8+math.pi/2*(8-4*centroid);expected=12*10*8-(1-math.pi/4)*8*18-area*length
 q.update(expected_volume=expected,native_volume_error=result['volume']-expected);assert abs(result['volume']-expected)<1e-7,(q,result)
(out/'summary.json').write_text(json.dumps(rows,indent=2)+'\n');print(json.dumps(rows,indent=2))
