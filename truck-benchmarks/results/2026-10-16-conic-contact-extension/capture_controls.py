from pathlib import Path
import sys,json,subprocess,os,shutil,math
root=Path.cwd();sys.path.insert(0,str(root/'audits/fillet-chamfer'))
from run_occt import isolated_display,execute,tcl_word
out=root/'target/native-conic-contact-extension';out.mkdir(exist_ok=False)
occt=(root/'../investigations/sequential-holes-2026-10-10/occt').resolve();plugin=root/'target/occt-cone-reference-plugin';sdk=root/'target/occt-sdk/usr/lib64'
with (out/'inputs.log').open('w') as log:subprocess.run([str(root/'target/conic-contact-native-inputs'),str(out)],env=dict(os.environ,LD_LIBRARY_PATH=str(sdk)),stdout=log,stderr=subprocess.STDOUT,check=True)
lines=['pload ALL','pload -BlendAudit BLENDAUDIT',f'source {tcl_word(root/"audits/fillet-chamfer/capture/blend_capture.tcl")}'];controls=[]
for source,scale in enumerate([1.,.2]):
 for mode,top,side,ref in [('equal',.25,.25,3),('unequal-top',.4,.2,3),('unequal-side',.4,.2,4)]:
  i=len(controls);name=f'bundle{i}'
  first,second=(top,side) if ref==3 else (side,top)
  request=f'{top*scale:.17g}' if mode=='equal' else f'{name}_{ref} {first*scale:.17g} {second*scale:.17g}'
  lines += [f'restore {tcl_word(out/f"input-{source}.brep")} {name}',f'explode {name}',f'audit_blend_output before{i} {name}_1',f'chamf result{i} {name}_1 {name}_2 {request}',f'audit_blend_output after{i} {name}_1']
  controls.append(dict(mode=mode,scale=scale,top=top,side=side))
lines.append('exit');script=out/'controls.tcl';script.write_text('\n'.join(lines)+'\n')
with isolated_display(out) as env:
 env.update(LD_LIBRARY_PATH=f'{plugin}:{sdk}',CASROOT=str(occt),CSF_BlendAuditDefaults=str(plugin),CSF_DrawPluginDefaults=str(occt/'src/DrawResources'),DRAWDEFAULT=str(occt/'src/DrawResources/DrawDefault'),CSF_TestScriptsPath=str(occt/'tests'),CSF_ShadersDirectory=str(occt/'src/Shaders'))
 log,code,timeout,elapsed=execute([shutil.which('DRAWEXE'),'-v','-f',str(script)],out,env,120);(out/'controls.log').write_text(log);assert code==0 and not timeout
captures=sorted((out/'capture').glob('*.input.json'));assert len(captures)==6
precise=subprocess.check_output([str(root/'target/precise-native-volume'),*[str(p).replace('.input.json','.result.brep') for p in captures]],env=dict(os.environ,LD_LIBRARY_PATH=str(sdk)),text=True)
(out/'precise-volumes.txt').write_text(precise)
precise=[list(map(float,line.split())) for line in precise.splitlines()]
assert len(precise)==6
for i,(p,c) in enumerate(zip(captures,controls)):
 result=json.loads(p.with_name(p.name.replace('.input.','.result.')).read_text());assert result['native_valid'] and result['solids']==1,(c,result)
 a=2.5;b=10.;R=5.;d=c['top'];e=c['side'];x0=-a;x1=-a+d
 circle=lambda x:(x*math.sqrt(R*R-x*x)+R*R*math.asin(x/R))/2
 weighted=lambda x:-(R*R-x*x)**1.5/3
 removed=e/d*((d-a)*(b*d-circle(x1)+circle(x0))-(b*(x1*x1-x0*x0)/2-weighted(x1)+weighted(x0)))
 initial=5*(5*math.pi*R*R/6+2*a*b-a*math.sqrt(R*R-a*a))
 expected=(initial-removed)*c['scale']**3
 assert abs(expected-precise[i][0])<1e-7,(c,expected,precise[i])
 assert (out/f'before{i}.result.brep').read_bytes()==(out/f'after{i}.result.brep').read_bytes()
 c.update(prefix=str(p)[:-len('.input.json')],expected_volume=expected,native_volume=result['volume'],adaptive_native_volume=precise[i][0],adaptive_reported_relative_error=precise[i][1])
 print(c,flush=True)
(out/'summary.json').write_text(json.dumps(controls,indent=2)+'\n')
