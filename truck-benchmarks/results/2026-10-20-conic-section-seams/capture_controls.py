from pathlib import Path
import sys,json,subprocess,os,shutil,math
root=Path.cwd();sys.path.insert(0,str(root/'audits/fillet-chamfer'))
from run_occt import isolated_display,execute,tcl_word
out=root/'target/native-conic-section';assert not (out/'capture').exists()
occt=(root/'../investigations/sequential-holes-2026-10-10/occt').resolve();plugin=root/'target/occt-cone-reference-plugin';sdk=root/'target/occt-sdk/usr/lib64'
lines=['pload ALL','pload -BlendAudit BLENDAUDIT',f'source {tcl_word(root/"audits/fillet-chamfer/capture/blend_capture.tcl")}'];rows=[]
for mode,top_axial,bottom_axial in [('equal',.5,.2),('unequal',.3,.1),('angle',.5/math.sqrt(3),.2/math.sqrt(3))]:
 i=len(rows);name=f'bundle{i}'
 request=f'{name}_4 .2' if mode=='equal' else (f'{name}_4 {name}_5 A .2 30' if mode=='angle' else f'{name}_4 {name}_5 .2 {bottom_axial}')
 lines += [f'restore {tcl_word(out/"input.brep")} {name}',f'explode {name}',f'audit_blend_output before{i} {name}_1',f'catch {{chamf result{i} {name}_1 {request}}} error{i}',f'audit_blend_output after{i} {name}_1']
 rows.append(dict(mode=mode,radius=5.,height=10.,theta=math.pi/2,top_radial=0.,top_axial=0.,bottom_planar=.2,bottom_axial=bottom_axial))
lines.append('exit');script=out/'controls.tcl';script.write_text('\n'.join(lines)+'\n')
with isolated_display(out) as env:
 env.update(LD_LIBRARY_PATH=f'{plugin}:{sdk}',CASROOT=str(occt),CSF_BlendAuditDefaults=str(plugin),CSF_DrawPluginDefaults=str(occt/'src/DrawResources'),DRAWDEFAULT=str(occt/'src/DrawResources/DrawDefault'),CSF_TestScriptsPath=str(occt/'tests'),CSF_ShadersDirectory=str(occt/'src/Shaders'))
 log,code,timeout,elapsed=execute([shutil.which('DRAWEXE'),'-v','-f',str(script)],out,env,120);(out/'controls.log').write_text(log);assert code==0 and not timeout
captures=sorted((out/'capture').glob('*.input.json'));assert len(captures)==len(rows)
for i,(p,q) in enumerate(zip(captures,rows)):
 result=json.loads(p.with_name(p.name.replace('.input.','.result.')).read_text());assert result['native_valid'] and result['solids']==1,(q,result)
 q['native_input_unchanged']=(out/f'before{i}.result.brep').read_bytes()==(out/f'after{i}.result.brep').read_bytes();q.update(prefix=str(p)[:-len('.input.json')],native=result)
 r,a=q['radius'],q['bottom_planar'];straight=q['bottom_axial']/a*(a/2*(a*math.sqrt(r*r-a*a)+r*r*math.asin(a/r))-(r**3-(r*r-a*a)**1.5)/3)
 top=q['theta']/2*q['top_axial']*(r*q['top_radial']-q['top_radial']**2/3);expected=q['theta']/2*r*r*q['height']-straight-top
 q.update(expected_volume=expected,native_volume_error=result['volume']-expected);assert abs(result['volume']-expected)<1e-7,(q,result)
(out/'summary.json').write_text(json.dumps(rows,indent=2)+'\n');print(json.dumps(rows,indent=2))
