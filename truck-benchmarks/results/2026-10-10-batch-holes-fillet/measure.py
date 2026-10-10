#!/usr/bin/env python3
"""Measure saved parent/new binaries on every action and direct OCCT case."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import statistics
import subprocess
import sys
import time

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[3]


def module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


comparison = module('comparison', ROOT/'truck-benchmarks/scripts/compare_occt.py')
actions = module('actions', ROOT/'truck-benchmarks/scripts/report_comparison.py')
parser = argparse.ArgumentParser(description=__doc__)
for name in ['before', 'after', 'occt', 'actions-before', 'actions-after', 'output']:
    parser.add_argument('--'+name, type=Path, required=True)
args = parser.parse_args()
out = args.output.resolve()
out.mkdir(exist_ok=False)
direct = out/'direct'
direct.mkdir()
action = out/'actions'
action.mkdir()
bins = {name: getattr(args, name).resolve() for name in ['before', 'after', 'occt']}
action_bins = {name: getattr(args, 'actions_'+name).resolve() for name in ['before', 'after']}
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
metadata = dict(parent=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
                created_utc=time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime()),
                cpus='0,1,2,3', workers=[1, 4], rounds=2, direct_samples=15, action_samples=20,
                tolerance=0.001, qa_tolerance=0.001, fuzzy=0,
                rustc=subprocess.check_output(['rustc', '-Vv'], text=True),
                platform=subprocess.check_output(['uname', '-a'], text=True),
                binaries={k: dict(path=str(p), sha256=sha(p)) for k, p in bins.items()},
                action_binaries={k: dict(path=str(p), sha256=sha(p)) for k, p in action_bins.items()},
                occt_version=subprocess.check_output([bins['occt'], '--version'], text=True).strip(),
                occt_libraries=subprocess.check_output(['ldd', bins['occt']], text=True))
source_paths = subprocess.check_output(['git', 'ls-files', '-co', '--exclude-standard', '*.rs', '*Cargo.toml'], cwd=ROOT, text=True).splitlines()
source_hashes = {p: sha(ROOT/p) for p in sorted(set(source_paths))}
(out/'sources.sha256').write_text(''.join(f'{h}  {p}\n' for p, h in source_hashes.items()))
patch = subprocess.check_output(['git', 'diff', '--binary'], cwd=ROOT)
for p in subprocess.check_output(['git', 'ls-files', '--others', '--exclude-standard', '*.rs'], cwd=ROOT, text=True).splitlines():
    patch += subprocess.run(['git', 'diff', '--no-index', '/dev/null', p], cwd=ROOT, stdout=subprocess.PIPE).stdout
(out/'source.patch').write_bytes(patch)
(out/'Cargo.lock').write_bytes((ROOT/'Cargo.lock').read_bytes())
env = {k: v for k, v in os.environ.items() if not k.startswith('DIVAN_') and k not in ['HOLE_PROFILE', 'VERIFY_STEPS', 'SPLIT_CYLINDER']}
env['OCCT_FUZZY'] = '0'
fixtures = out/'fixtures'
subprocess.run([bins['after'], '--export-threads', fixtures], env=env, check=True, cwd=ROOT)
metadata['fixtures'] = {p.name: sha(p) for p in fixtures.iterdir()}
(out/'metadata.json').write_text(json.dumps(metadata, indent=2)+'\n')
records = []
for worker in [1, 4]:
    for round_ in [0, 1, 2]:
        for case in comparison.CASES:
            for engine in (['occt', 'after', 'before'] if round_ == 2 else ['before', 'after', 'occt']):
                count = 0 if round_ == 0 else 15
                log = direct/f'r{round_}-{engine}-{worker}t-{case.replace("/", "-")}.log'
                cmd = ['taskset', '-c', '0,1,2,3', bins[engine], case, str(count), '0.001', '0.001', fixtures]
                execution = comparison.command(cmd, log, dict(env, RAYON_NUM_THREADS=str(worker)), 180)
                record = dict(engine=engine, case=case, workers=worker, round=round_,
                              **comparison.parse_result(log.read_text(), count, execution))
                records.append(record)
                with (direct/'records.jsonl').open('a') as stream:
                    stream.write(json.dumps(record)+'\n')
                print('Direct', round_, engine, worker, case, record['status'], flush=True)
                assert record['status'] == 'pass', record
rows = []
for worker in [1, 4]:
    for case in comparison.CASES:
        row = dict(case=case, workers=worker)
        volumes = []
        for engine in bins:
            runs = [r for r in records if (r['case'], r['workers'], r['engine']) == (case, worker, engine) and r['round']]
            assert len(runs) == 2
            medians = [statistics.median(r['timing']['samples_ms']) for r in runs]
            row[engine] = statistics.median(medians)
            row[engine+'_rounds'] = medians
            volumes.extend(r['qa']['volume']/r['qa']['expected_volume'] for r in runs)
        assert max(volumes)-min(volumes) <= .002, row
        row['speedup'] = row['before']/row['after']
        row['occt_over_after'] = row['occt']/row['after']
        rows.append(row)
(direct/'summary.json').write_text(json.dumps(rows, indent=2)+'\n')
action_records = []
for round_ in [1, 2]:
    for worker in [1, 4]:
        for engine in (['before', 'after'] if round_ == 1 else ['after', 'before']):
            log = action/f'r{round_}-{engine}-{worker}t.txt'
            cmd = ['taskset', '-c', '0,1,2,3', action_bins[engine], '--bench', '--color', 'never',
                   '--sample-count', '20', '--sample-size', '1', '--max-time', '60']
            execution = comparison.command(cmd, log, dict(env, RAYON_NUM_THREADS=str(worker)), 180)
            record = dict(engine=engine, workers=worker, round=round_, **execution)
            assert execution['exit_code'] == 0, record
            record['medians_ms'] = actions.divan(log)
            assert len(record['medians_ms']) == 72, record
            action_records.append(record)
            (action/'records.json').write_text(json.dumps(action_records, indent=2)+'\n')
            print('Actions', round_, engine, worker, '72 passed', flush=True)
action_rows = []
for worker in [1, 4]:
    for case in sorted(action_records[0]['medians_ms']):
        row = dict(case=case, workers=worker)
        for engine in action_bins:
            medians = [r['medians_ms'][case] for r in action_records if r['workers'] == worker and r['engine'] == engine]
            row[engine] = statistics.median(medians)
            row[engine+'_rounds'] = medians
        row['speedup'] = row['before']/row['after']
        action_rows.append(row)
(action/'summary.json').write_text(json.dumps(action_rows, indent=2)+'\n')
assert all(sha(ROOT/p) == h for p, h in source_hashes.items()), 'sources changed during measurement'
for key, paths in [('binaries', bins), ('action_binaries', action_bins)]:
    assert all(sha(p) == metadata[key][k]['sha256'] for k, p in paths.items())
print('Finished: 360 direct records, 40 volume comparisons, 72 actions in each of 8 runs; hashes unchanged.', flush=True)
