from pathlib import Path
import sys,json,subprocess,os,shutil,math
import mpmath as mp
root=Path.cwd();sys.path.insert(0,str(root/'audits/fillet-chamfer'))
from run_occt import isolated_display,execute,tcl_word
out=root/'target/native-curved-chamfer-terminals';out.mkdir(exist_ok=False)
occt=(root/'../investigations/sequential-holes-2026-10-10/occt').resolve();plugin=root/'target/occt-cone-reference-plugin';sdk=root/'target/occt-sdk/usr/lib64'
with (out/'inputs.log').open('w') as log:subprocess.run([str(root/'target/curved-terminal-native-inputs'),str(out)],env=dict(os.environ,LD_LIBRARY_PATH=str(sdk)),stdout=log,stderr=subprocess.STDOUT,check=True)
lines=['pload ALL','pload -BlendAudit BLENDAUDIT',f'source {tcl_word(root/"audits/fillet-chamfer/capture/blend_capture.tcl")}'];controls=[]
for source,scale in enumerate([1.,.2]):
 for mode,planar,chord in [('native-equal',.2,.2),('unequal',.1,.2),('kernel-equal',.2,2*math.sin(math.atan(.2)/2))]:
  i=len(controls);name=f'bundle{i}';request=f'{planar*scale:.17g}' if mode=='native-equal' else f'{name}_3 {planar*scale:.17g} {chord*scale:.17g}'
  lines += [f'restore {tcl_word(out/f"input-{source}.brep")} {name}',f'explode {name}',f'audit_blend_output before{i} {name}_1',f'chamf result{i} {name}_1 {name}_2 {request}',f'audit_blend_output after{i} {name}_1']
  controls.append(dict(mode=mode,scale=scale,planar=planar,chord=chord))
lines.append('exit');script=out/'controls.tcl';script.write_text('\n'.join(lines)+'\n')
with isolated_display(out) as env:
 env.update(LD_LIBRARY_PATH=f'{plugin}:{sdk}',CASROOT=str(occt),CSF_BlendAuditDefaults=str(plugin),CSF_DrawPluginDefaults=str(occt/'src/DrawResources'),DRAWDEFAULT=str(occt/'src/DrawResources/DrawDefault'),CSF_TestScriptsPath=str(occt/'tests'),CSF_ShadersDirectory=str(occt/'src/Shaders'))
 log,code,timeout,elapsed=execute([shutil.which('DRAWEXE'),'-v','-f',str(script)],out,env,120);(out/'controls.log').write_text(log);assert code==0 and not timeout
captures=sorted((out/'capture').glob('*.input.json'));assert len(captures)==6
mp.mp.dps=60
for i,(p,c) in enumerate(zip(captures,controls)):
 result=json.loads(p.with_name(p.name.replace('.input.','.result.')).read_text());assert result['native_valid'] and result['solids']==1,(c,result)
 a=1-mp.mpf(str(c['planar']));D=mp.mpf(str(c['chord']));psi=2*mp.asin(D/2);alpha=mp.atan(mp.sin(psi)/(mp.cos(psi)-a))
 def removed(r):
  t=-a*mp.cos(alpha)+mp.sqrt(r*r-a*a*mp.sin(alpha)**2);y=a+t*mp.cos(alpha);z=t*mp.sin(alpha)
  return (r*r*mp.atan2(z,y)-a*z)/2
 initial=mp.pi/4*(1+mp.mpf('.2')/3*(1+mp.mpf('.95')+mp.mpf('.95')**2)-mp.mpf('1.2')*mp.mpf('.25')**2)
 expected=(initial-removed(1)-mp.quad(lambda x:removed(1-(x-1)/4),[1,mp.mpf('1.2')]))*mp.mpf(str(c['scale']))**3
 assert abs(float(expected)-result['volume'])<1e-7,(c,float(expected),result)
 assert (out/f'before{i}.result.brep').read_bytes()==(out/f'after{i}.result.brep').read_bytes()
 c.update(prefix=str(p)[:-len('.input.json')],kernel_tangent=float(mp.tan(psi)),expected_volume=float(expected),expected_volume_decimal=mp.nstr(expected,50),native_volume=result['volume'])
 print(c,flush=True)
(out/'summary.json').write_text(json.dumps(controls,indent=2)+'\n')
