#!/usr/bin/env python3
"""Repeat the action family that exceeded 10% slowdown in the full run."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import statistics
import subprocess
import sys
sys.dont_write_bytecode=True
ROOT=Path(__file__).resolve().parents[3]
OUT=Path(__file__).resolve().parent
DEST=OUT/'data/rechecks'
DEST.mkdir(exist_ok=False)
spec=importlib.util.spec_from_file_location('report_comparison',ROOT/'truck-benchmarks/scripts/report_comparison.py')
module=importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
metadata=json.loads((OUT/'data/metadata.json').read_text())
bins={engine:entry['path'] for engine,entry in metadata['action_binaries'].items()}
for engine,path in bins.items():
    assert hashlib.sha256(Path(path).read_bytes()).hexdigest()==metadata['action_binaries'][engine]['sha256']
env={k:v for k,v in os.environ.items() if not k.startswith('DIVAN_')}
env['RAYON_NUM_THREADS']='4'
records=[]
for family in ['cylinder_tolerance']:
    for round in range(1,6):
        for engine in (['before','after'] if round%2 else ['after','before']):
            name=f'{family}-r{round}-{engine}.txt'
            command=['taskset','-c','0,1,2,3',bins[engine],'--bench',family,'--color','never','--sample-count','500','--sample-size','1','--max-time','60']
            with (DEST/name).open('w') as log:
                result=subprocess.run(command,cwd=ROOT,env=env,stdout=log,stderr=subprocess.STDOUT)
            record=dict(family=family,round=round,engine=engine,workers=4,exit_code=result.returncode,command=command,log=name)
            assert result.returncode==0,record
            record['medians_ms']=module.divan(DEST/name)
            assert len(record['medians_ms'])==3,record
            records.append(record)
            (DEST/'records.json').write_text(json.dumps(records,indent=2)+'\n')
            print(family,round,engine,record['medians_ms'],flush=True)
rows=[]
for case in sorted({case for record in records for case in record['medians_ms']}):
    row=dict(case=case,workers=4)
    for engine in ['before','after']:
        values=[r['medians_ms'][case] for r in records if r['engine']==engine and case in r['medians_ms']]
        assert len(values)==5
        row[engine]=statistics.median(values)
        row[engine+'_rounds']=values
    row['speedup']=row['before']/row['after']
    rows.append(row)
(DEST/'summary.json').write_text(json.dumps(rows,indent=2)+'\n')
print(json.dumps(rows,indent=2),flush=True)
