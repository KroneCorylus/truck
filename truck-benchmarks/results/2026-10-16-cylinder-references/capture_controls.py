from pathlib import Path
import sys,json,shutil,subprocess,os,math
root=Path.cwd();sys.path.insert(0,str(root/'audits/fillet-chamfer'))
from run_occt import isolated_display,execute,tcl_word
out=(root/sys.argv[2]).resolve();out.mkdir(exist_ok=False)
occt=(root/'../investigations/sequential-holes-2026-10-10/occt').resolve();plugin=(root/sys.argv[1]).resolve();sdk=root/'target/occt-sdk/usr/lib64';draw=shutil.which('DRAWEXE')
with (out/'inputs.log').open('w') as log:subprocess.run([str(root/'target/cylinder-reference-native-inputs'),str(out)],env=dict(os.environ,LD_LIBRARY_PATH=str(sdk)),stdout=log,stderr=subprocess.STDOUT,check=True)
script=out/'controls.tcl'
lines=['pload ALL','pload -BlendAudit BLENDAUDIT',f'source {tcl_word(root/"audits/fillet-chamfer/capture/blend_capture.tcl")}']
for i,(a,b) in enumerate([(.3,.7),(.3,'A'),(.09,.02)]):
 lines.extend([f'restore {tcl_word(out/f"input-{i}.brep")} bundle{i}',f'puts [explode bundle{i}]',f'audit_blend_output before{i} bundle{i}_1',f'if {{[catch {{chamf result{i} bundle{i}_1 bundle{i}_2 bundle{i}_4 {('A '+str(a)+' 60') if b=='A' else (str(a)+' '+str(b))}}} message]}} {{puts $message;exit 3}}',f'audit_blend_output after{i} bundle{i}_1'])
lines.append('exit');script.write_text('\n'.join(lines)+'\n')
with isolated_display(out) as env:
 env.update(LD_LIBRARY_PATH=f'{plugin}:{sdk}',CASROOT=str(occt),CSF_BlendAuditDefaults=str(plugin),CSF_DrawPluginDefaults=str(occt/'src/DrawResources'),DRAWDEFAULT=str(occt/'src/DrawResources/DrawDefault'),CSF_TestScriptsPath=str(occt/'tests'),CSF_ShadersDirectory=str(occt/'src/Shaders'))
 log,code,timeout,elapsed=execute([draw,'-v','-f',str(script)],out,env,120);(out/'controls.log').write_text(log);assert code==0 and not timeout,(code,timeout,log[-3000:])
 captures=sorted((out/'capture').glob('*.input.json'));assert len(captures)==3,len(captures)
 summary=[]
 for i,(p,(radial,vertical,scale)) in enumerate(zip(captures,[(.7,.3,1),(.3*math.tan(math.pi/3),.3,1),(.2,.9,.1)])):
  task=json.loads(p.read_text());result=json.loads(p.with_name(p.name.replace('.input.','.result.')).read_text())
  assert task['schema_version']==2 and task['operation']=='chamfer'
  assert len(task['selection'])==1 and len(task['selection'][0]['contour_edges'])==1
  assert result['native_valid'] and result['solids']==1,result
  expected=(math.pi*25*10/2-math.pi/2*(5*radial*vertical-radial**2*vertical/3))*scale**3
  assert abs(result['volume']-expected)<1e-7,(result['volume'],expected)
  assert (out/f'before{i}.result.brep').read_bytes()==(out/f'after{i}.result.brep').read_bytes()
  summary.append(dict(prefix=str(p)[:-len('.input.json')],radial=radial,vertical=vertical,scale=scale,expected_volume=expected,native=result))
 (out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary,indent=2))

for p in captures:
 task=json.loads(p.read_text());reference=task['selection'][0].get('reference_cylinder')
 assert reference is not None, 'Missing reference_cylinder for '+str(p)
 assert task['selection'][0]['reference_plane'] is None
 scale=.1 if p.name.startswith('003.') else 1.
 assert abs(reference['radius']-5*scale)<1e-12
 assert abs(abs(reference['axis'][2])-1)<1e-12
 assert max(abs(v) for v in reference['origin'][:2])<1e-12
print('Native cylinder descriptors, side distances, angle, scope and input immutability verified')
