#!/usr/bin/env python3
"""Compare saved parent/new Rust runners and a direct OCCT runner on unchanged fixtures."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import statistics
import subprocess

ROOT = Path(__file__).resolve().parents[3]
spec = importlib.util.spec_from_file_location('comparison', ROOT / 'truck-benchmarks/scripts/compare_occt.py')
comparison = importlib.util.module_from_spec(spec)
spec.loader.exec_module(comparison)
parser = argparse.ArgumentParser()
parser.add_argument('--before', type=Path, required=True)
parser.add_argument('--after', type=Path, required=True)
parser.add_argument('--occt', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
out = args.output.resolve()
out.mkdir(parents=True, exist_ok=False)
bins = {name: str(getattr(args, name).resolve()) for name in ('before', 'after', 'occt')}
metadata = dict(binaries=bins, sha256={k: hashlib.sha256(Path(v).read_bytes()).hexdigest() for k,v in bins.items()},
                revision=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
                cpus='0,1,2,3', samples=5, rounds=2, tolerance=0.001,
                rustc=subprocess.check_output(['rustc','-Vv'],text=True),
                kernel=subprocess.check_output(['uname','-a'],text=True).strip(),
                cpu=next(line.split(':',1)[1].strip() for line in Path('/proc/cpuinfo').read_text().splitlines() if line.startswith('model name')))
(out/'metadata.json').write_text(json.dumps(metadata,indent=2)+'\n')
records = []

def run(engine, case, workers=1, round=0, samples=0):
    name=f'{engine}-{case.replace("/","-")}-{workers}t-r{round}'
    env=os.environ.copy()
    for key in ['HOLE_PROFILE', 'VERIFY_STEPS', 'SPLIT_CYLINDER']:
        env.pop(key,None)
    env['RAYON_NUM_THREADS']=str(workers)
    qa=0.001
    if case.startswith('thin_wall/'):
        qa=max(1e-6,float(case.split('/')[1])/1000.)
    if case.startswith('near_tangent/'):
        qa=max(1e-6,float(case.split('/')[1])/10000.)
    command=['taskset','-c','0,1,2,3',bins[engine],case,str(samples),'0.001',str(qa),str(out)]
    try:
        result=subprocess.run(command,cwd=ROOT,env=env,text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,timeout=180)
        code=result.returncode
        log=result.stdout
    except subprocess.TimeoutExpired as error:
        code=124
        log=(error.stdout or b'').decode()
    path=out/(name+'.log')
    path.write_text(log)
    record=dict(engine=engine,case=case,workers=workers,round=round,**comparison.parse_result(log,samples,dict(status='complete' if code==0 else 'failed', exit_code=code, command=command, log=path.name)))
    records.append(record)
    (out/'records.json').write_text(json.dumps(records,indent=2)+'\n')
    values=(record.get('timing') or {}).get('samples_ms',[])
    print(name,record['status'],round_value(statistics.median(values)) if values else '-',flush=True)
    if record['status']!='pass':
        raise RuntimeError(f'{name} did not pass: {record}')

def round_value(value):
    return float(f'{value:.4f}')

# All unchanged fixtures validate the optimized kernel before timing any workload.
for case in comparison.CASES:
    run('after',case)
for workers in [1,4]:
    for n in [1,10,30,100]:
        case=f'holes_sequential/{n}'
        for engine in (['before','occt'] if workers==1 else ['before','after','occt']):
            run(engine,case,workers)
        for round in [1,2]:
            for engine in (['before','after','occt'] if round==1 else ['occt','after','before']):
                run(engine,case,workers,round,5)
# Adjacent actions and an unsupported batch check downstream effects and generic overhead.
for case in ['holes_batch/100','thin_wall/0.01','fillet_bore/0.5','mesh_plate/100','knurl/16']:
    run('before',case)
    for round in [1,2]:
        for engine in (['before','after'] if round==1 else ['after','before']):
            run(engine,case,1,round,5)
rows=[]
for case,workers in dict.fromkeys((r['case'],r['workers']) for r in records if r['round']):
    row=dict(case=case,workers=workers)
    volumes=[]
    for engine in ['before','after','occt']:
        runs=[r for r in records if (r['case'],r['workers'],r['engine'])==(case,workers,engine) and r['round']]
        if runs:
            values=[statistics.median(r['timing']['samples_ms']) for r in runs]
            row[engine]=statistics.median(values)
            volumes.extend(r['qa']['volume']/r['qa']['expected_volume'] for r in runs)
    assert max(volumes)-min(volumes)<=0.002,(case,volumes)
    row['speedup']=row['before']/row['after']
    rows.append(row)
(out/'summary.json').write_text(json.dumps(rows,indent=2)+'\n')
print(json.dumps(rows,indent=2),flush=True)
