from pathlib import Path
import sys,json,subprocess,os,shutil,math
root=Path.cwd();sys.path.insert(0,str(root/'audits/fillet-chamfer'))
from run_occt import isolated_display,execute,tcl_word
out=root/'target/native-longitudinal-offset-controls';out.mkdir(exist_ok=False)
occt=(root/'../investigations/sequential-holes-2026-10-10/occt').resolve();plugin=root/'target/occt-cone-reference-plugin';sdk=root/'target/occt-sdk/usr/lib64'
with (out/'inputs.log').open('w') as log:subprocess.run([str(root/'target/longitudinal-offset-inputs'),str(out)],env=dict(os.environ,LD_LIBRARY_PATH=str(sdk)),stdout=log,stderr=subprocess.STDOUT,check=True)
lines=['pload ALL','pload -BlendAudit BLENDAUDIT',f'source {tcl_word(root/"audits/fillet-chamfer/capture/blend_capture.tcl")}']
controls=[]
for source in range(6):
 for mode,ref in [('equal',3),('unequal',3),('unequal',4)]:
  i=len(controls);name=f'bundle{i}'
  request='.5' if mode=='equal' else (f'{name}_{ref} .7 1.5' if ref==3 else f'{name}_{ref} 1.5 .7')
  lines += [f'restore {tcl_word(out/f"input-{source}.brep")} {name}',f'explode {name}',f'audit_blend_output before{i} {name}_1',f'chamf result{i} {name}_1 {name}_2 {request}',f'audit_blend_output after{i} {name}_1']
  controls.append(dict(mode=mode,reference='plane' if ref==3 else 'cylinder',convex=source<3,offset=[-2.,0.,2.][source%3]))
lines.append('exit');script=out/'controls.tcl';script.write_text('\n'.join(lines)+'\n')
with isolated_display(out) as env:
 env.update(LD_LIBRARY_PATH=f'{plugin}:{sdk}',CASROOT=str(occt),CSF_BlendAuditDefaults=str(plugin),CSF_DrawPluginDefaults=str(occt/'src/DrawResources'),DRAWDEFAULT=str(occt/'src/DrawResources/DrawDefault'),CSF_TestScriptsPath=str(occt/'tests'),CSF_ShadersDirectory=str(occt/'src/Shaders'))
 log,code,timeout,elapsed=execute([shutil.which('DRAWEXE'),'-v','-f',str(script)],out,env,120);(out/'controls.log').write_text(log);assert code==0 and not timeout
captures=sorted((out/'capture').glob('*.input.json'));assert len(captures)==18
for i,(p,c) in enumerate(zip(captures,controls)):
 result=json.loads(p.with_name(p.name.replace('.input.','.result.')).read_text());assert result['native_valid'] and result['solids']==1
 radius=5.;height=10.;h=c['offset'];x=math.sqrt(radius**2-h*h)
 a,D=(.5,.5) if c['mode']=='equal' else (.7,1.5)
 psi=2*math.asin(D/(2*radius));theta=math.asin(h/radius)
 px=x+(-1 if c['convex'] else 1)*a
 qx=radius*math.cos(theta+psi);qy=radius*math.sin(theta+psi)
 removed=abs(x*h-h*px+px*qy-h*qx-radius**2*psi)/2
 area=radius**2*math.acos(h/radius)-h*x
 expected=((area if c['convex'] else 16*(8-h)-area)-removed)*height
 assert abs(expected-result['volume'])<1e-7,(c,expected,result)
 assert (out/f'before{i}.result.brep').read_bytes()==(out/f'after{i}.result.brep').read_bytes()
 c.update(prefix=str(p)[:-len('.input.json')],planar_inset=a,cylinder_chord=2*radius*math.sin(psi/2),kernel_tangent=radius*math.tan(psi),expected_volume=expected,native_volume=result['volume'])
 print(c,flush=True)
(out/'summary.json').write_text(json.dumps(controls,indent=2)+'\n')
