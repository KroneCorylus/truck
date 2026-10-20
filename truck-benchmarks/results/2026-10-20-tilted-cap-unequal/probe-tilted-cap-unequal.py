from pathlib import Path
import json,shutil,subprocess,hashlib
root=Path.cwd();out=root/'target/tilted-cap-unequal-parent';out.mkdir(exist_ok=False);rows=[]
for q in json.load(open('target/native-tilted-cap-unequal/summary.json')):
 prefix=out/q['mode']
 for ext in ['input.json','input.step']:shutil.copyfile(q['prefix']+'.'+ext,str(prefix)+'.'+ext)
 r=subprocess.run(['target/release/examples/replay_occt_blend',str(prefix)],capture_output=True,text=True);(out/(q['mode']+'.log')).write_text(r.stdout+r.stderr)
 result=json.load(open(str(prefix)+'.truck.json'))['result'];rows.append(dict(mode=q['mode'],result=result));print(q['mode'],result.get('status'),result.get('error'),flush=True)
(out/'summary.json').write_text(json.dumps(rows,indent=2)+'\n');(out/'revision.txt').write_bytes(subprocess.check_output(['git','rev-parse','HEAD']))
