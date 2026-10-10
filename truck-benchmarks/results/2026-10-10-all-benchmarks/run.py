#!/usr/bin/env python3
"""Run all Truck action benchmarks and the full direct OCCT comparison sequentially."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import statistics
import subprocess
import time

ROOT = Path(__file__).resolve().parents[3]
OUT = Path(__file__).resolve().parent
PARENT = ROOT.parent/'truck'
ACTION = OUT/'actions'
ACTION.mkdir(exist_ok=False)
(OUT/'.gitignore').write_text('!*.json\n!*.jsonl\n!Cargo.lock\n__pycache__/\n')
spec=importlib.util.spec_from_file_location('report_comparison',ROOT/'truck-benchmarks/scripts/report_comparison.py')
module=importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
env=os.environ.copy()
for key in list(env):
    if key.startswith('DIVAN_') or key in ['HOLE_PROFILE','VERIFY_STEPS','SPLIT_CYLINDER','OCCT_FUZZY']:
        env.pop(key,None)

def run(command, log, cwd=ROOT, environment=env):
    print('Running:', ' '.join(map(str,command)),flush=True)
    start=time.monotonic()
    with log.open('w') as stream:
        result=subprocess.run(command,cwd=cwd,env=environment,stdout=stream,stderr=subprocess.STDOUT)
    record=dict(command=list(map(str,command)),cwd=str(cwd),exit_code=result.returncode,seconds=time.monotonic()-start,log=str(log.relative_to(OUT)))
    print(record,flush=True)
    return record

metadata=dict(created_utc=time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime()),cpus='0,1,2,3',workers=[1,4],samples=20,rounds=2,
    parent_revision=subprocess.check_output(['git','rev-parse','HEAD'],cwd=PARENT,text=True).strip(),
    revision=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
    rustc=subprocess.check_output(['rustc','-Vv'],text=True),builds=[],binaries={})
assert not subprocess.check_output(['git','status','--porcelain'],cwd=PARENT,text=True),'parent checkout must be clean'
assert (ROOT/'Cargo.lock').read_bytes()==(PARENT/'Cargo.lock').read_bytes(),'different dependency locks'
bin_dir=ROOT/'target/all-benchmarks'
bin_dir.mkdir(exist_ok=True)
bins={}
# Both action binaries are built before any measurement starts.
for engine,root in [('before',PARENT),('after',ROOT)]:
    log=ACTION/f'build-{engine}.log'
    execution=run(['cargo','bench','--locked','-p','truck-benchmarks','--bench','actions','--no-run','--message-format=json'],log,root)
    metadata['builds'].append(execution)
    assert execution['exit_code']==0,execution
    artifacts=[json.loads(line) for line in log.read_text().splitlines() if line.startswith('{')]
    artifact=next(r for r in artifacts if r.get('reason')=='compiler-artifact' and r.get('target',{}).get('name')=='actions' and r.get('executable'))
    binary=bin_dir/f'actions-{engine}'
    shutil.copy2(artifact['executable'],binary)
    bins[engine]=binary
    metadata['binaries'][engine]=dict(path=str(binary),sha256=hashlib.sha256(binary.read_bytes()).hexdigest())
(OUT/'actions/metadata.json').write_text(json.dumps(metadata,indent=2)+'\n')
(OUT/'actions/Cargo.lock').write_bytes((ROOT/'Cargo.lock').read_bytes())
records=[]
for round in [1,2]:
    for worker in [1,4]:
        for engine in (['before','after'] if round==1 else ['after','before']):
            log=ACTION/f'r{round}-{engine}-{worker}t.txt'
            command=['taskset','-c','0,1,2,3',str(bins[engine]),'--bench','--color','never','--sample-count','20','--sample-size','1','--max-time','60']
            execution=run(command,log,environment=dict(env,RAYON_NUM_THREADS=str(worker)))
            record=dict(engine=engine,workers=worker,round=round,**execution)
            try:
                record['medians_ms']=module.divan(log)
            except (AssertionError,ValueError) as error:
                record['parse_error']=str(error)
            records.append(record)
            (ACTION/'records.json').write_text(json.dumps(records,indent=2)+'\n')
            print('Action cases:',len(record.get('medians_ms',{})),flush=True)
if all(r['exit_code']==0 and 'medians_ms' in r for r in records):
    keys=set(records[0]['medians_ms'])
    assert all(set(r['medians_ms'])==keys for r in records),'case coverage changed'
    rows=[]
    for worker in [1,4]:
        for case in sorted(keys):
            row=dict(case=case,workers=worker)
            for engine in ['before','after']:
                values=[r['medians_ms'][case] for r in records if r['workers']==worker and r['engine']==engine]
                row[engine]=statistics.median(values)
                row[engine+'_rounds']=values
            row['speedup']=row['before']/row['after']
            rows.append(row)
    (ACTION/'summary.json').write_text(json.dumps(rows,indent=2)+'\n')
# The full direct runner builds its own uninstrumented comparison executable and captures source provenance.
occt=PARENT/'target/occt-sdk/usr'
execution=run(['python3','truck-benchmarks/scripts/compare_occt.py','--output',str(OUT/'occt'),
    '--samples','15','--rounds','2','--workers','1,4','--cpus','0,1,2,3',
    '--occt-include',str(occt/'include/opencascade'),'--occt-lib',str(occt/'lib64')],OUT/'occt-run.log')
(OUT/'execution.json').write_text(json.dumps(dict(actions=records,occt=execution),indent=2)+'\n')
assert execution['exit_code']==0,execution
assert all(r['exit_code']==0 and 'medians_ms' in r for r in records),'action run failed'
print('Finished full benchmark run.',flush=True)
