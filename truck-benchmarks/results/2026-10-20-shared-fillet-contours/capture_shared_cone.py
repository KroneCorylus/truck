from pathlib import Path
import sys,json,subprocess,os,shutil,math
root=Path.cwd();sys.path.insert(0,str(root/'audits/fillet-chamfer'))
from run_occt import isolated_display,execute,tcl_word
out=root/'target/native-shared-cone-fillet';out.mkdir(exist_ok=True);assert not (out/'capture').exists()
occt=(root/'../investigations/sequential-holes-2026-10-10/occt').resolve();plugin=root/'target/occt-cone-reference-plugin';sdk=root/'target/occt-sdk/usr/lib64'
source=root/'target/occt-capture-conic-section-seams/feat/featrf/A4/capture/001.input.brep';shutil.copy2(source,out/'source.brep')
subprocess.run([str(root/'target/shared-cone-fillet-input'),str(out/'input.brep'),str(source)],check=True)
lines=['pload ALL','pload -BlendAudit BLENDAUDIT',f'source {tcl_word(root/"audits/fillet-chamfer/capture/blend_capture.tcl")}',f'restore {tcl_word(out/"input.brep")} bundle','explode bundle','audit_blend_output before bundle_1','catch {blend result bundle_1 .3 bundle_2 .3 bundle_3} error','audit_blend_output after bundle_1','exit'];script=out/'controls.tcl';script.write_text('\n'.join(lines)+'\n')
with isolated_display(out) as env:
 env.update(LD_LIBRARY_PATH=f'{plugin}:{sdk}',CASROOT=str(occt),CSF_BlendAuditDefaults=str(plugin),CSF_DrawPluginDefaults=str(occt/'src/DrawResources'),DRAWDEFAULT=str(occt/'src/DrawResources/DrawDefault'),CSF_TestScriptsPath=str(occt/'tests'),CSF_ShadersDirectory=str(occt/'src/Shaders'))
 log,code,timeout,elapsed=execute([shutil.which('DRAWEXE'),'-v','-f',str(script)],out,env,120);(out/'controls.log').write_text(log);assert code==0 and not timeout
captures=list((out/'capture').glob('*.input.json'));assert len(captures)==1;p=captures[0];result=json.loads(p.with_name(p.name.replace('.input.','.result.')).read_text());assert result['native_valid'] and result['solids']==1
r,k=.3,1/6;n=math.hypot(1,k);h=r-r*k/n
P=lambda c,t:r*(c*c*math.sin(t)+c*r*(t+math.sin(2*t)/2)+r*r*(math.sin(t)-math.sin(t)**3/3))
c=2+r*(n-k);rho=c-r/n;added=-math.pi*(h*(4+2*rho+rho*rho)/3+P(c,-math.pi/2)-P(c,math.atan2(-k,-1)))
c=1.5-r*(n-k);rho=c+r/n;removed=math.pi*(h*(2.25+1.5*rho+rho*rho)/3-P(c,math.pi/2)+P(c,math.atan(k)))
expected=56.25*math.pi+added-removed;assert abs(expected-176.76295442022382)<1e-12;assert abs(result['volume']-expected)<1e-7
q=dict(mode='stack',fillet_radius=r,expected_volume=expected,native_volume_error=result['volume']-expected,native_input_unchanged=(out/'before.result.brep').read_bytes()==(out/'after.result.brep').read_bytes(),prefix=str(p)[:-len('.input.json')],native=result,added_volume=added,removed_volume=removed)
(out/'summary.json').write_text(json.dumps(q,indent=2)+'\n');print(json.dumps(q,indent=2))
