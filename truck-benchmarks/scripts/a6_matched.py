#!/usr/bin/env python3
"""Prepare/build/smoke three explicit chosen-shape captures; timed rounds require a reserved serial slot.

Use a fresh --output directory for each campaign, running each stage once. Failed
attempts are retained; retries require a new campaign. Smoke and timing evidence
live in smoke/ and time/, including time/summary.json only after a successful run.
"""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import shutil
import statistics
import subprocess

ROOT = Path(__file__).resolve().parents[2]
BASE = ROOT / 'target/occt-capture-exact-knots-2026-10-24'
CASES = [f'chamfer/equal_dist_complex/{c}' for c in ('A6','A8','A9')]
SDK = ROOT / 'target/occt-sdk/usr'
ENV = dict(os.environ, LD_LIBRARY_PATH=str(SDK / 'lib64'), RAYON_NUM_THREADS='1', OMP_NUM_THREADS='1', CARGO_BUILD_JOBS='8')

def digest(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()

def save(path, value, exclusive=False):
    with path.open('x' if exclusive else 'w') as stream:
        stream.write(json.dumps(value, indent=2) + '\n')

def run(cmd, log, env=ENV):
    with log.open('x') as f:
        subprocess.run(list(map(str, cmd)), cwd=ROOT, env=env, stdout=f, stderr=subprocess.STDOUT, check=True, timeout=600)

def prepare(out):
    out.mkdir(parents=True, exist_ok=False)
    records = []
    for key in CASES:
        src = BASE / key / 'capture/001'
        folder = out / key.replace('/', '-')
        folder.mkdir(parents=True, exist_ok=True)
        for suffix in ('.input.json', '.input.step', '.input.brep', '.result.json', '.result.brep'):
            shutil.copyfile(str(src)+suffix, folder / ('001'+suffix))
        shutil.copyfile(ROOT/'target/a6-preference-kernel-20261104/annotated'/key.split('/')[-1]/'001.preferences.json', folder/'001.preferences.json')
        task = json.loads((folder/'001.input.json').read_text())
        if task['operation'] == 'fillet':
            assert task['native_settings'] == dict(representation='rational',angular_tolerance=.01,spine_tolerance=.0001,parametric_tolerance=.00001,approximation_3d_tolerance=.0001,deflection=.001,continuity='C1',continuity_angle=.01)
        rows = []
        for s in task['selection']:
            assert s.get('native_applied', True), 'Ignored native assignment needs explicit handling'
            mode = {'equal-distance':1,'two-distances':2,'distance-angle':3}.get(s.get('mode'),0)
            if mode == 0: a,b=s['radius'],0
            elif mode == 1: a,b=s['distance'],0
            elif mode == 2: a,b=s['distances']
            else: a,b=s['distance'],math.radians(s['angle_degrees'])
            plane = s.get('reference_plane') or dict(origin=[0]*3,normal=[0]*3)
            assert mode < 2 or s.get('reference_plane'), 'Unsupported reference surface'
            row = [mode,a,b,*plane['origin'],*plane['normal'],len(s['points'])]
            row += [v for p in s['points'] for v in p]
            rows.append(' '.join(map(str,row)))
        (folder/'native.config').write_text(str(len(rows))+'\n'+'\n'.join(rows)+'\n')
        records.append(dict(case=key,folder=str(folder),hashes={p.name:digest(p) for p in folder.iterdir() if p.is_file()},comparison_status='FAILED' if key.endswith(('H1','F9')) else 'limited-pass-retained'))
    save(out/'inputs.json',records,exclusive=True)
    return records

def sources():
    paths = [ROOT/'Cargo.lock',ROOT/'Cargo.toml']
    for crate in ROOT.glob('truck-*'):
        paths += list(crate.glob('src/**/*.rs')) + list(crate.glob('Cargo.toml'))
    paths += [ROOT/'truck-shapeops/examples/replay_occt_blend.rs',Path(__file__).resolve(),ROOT/'truck-benchmarks/examples/a6_matched.rs',ROOT/'truck-benchmarks/occt/nov3_matched.cpp']
    return {str(p.relative_to(ROOT)):digest(p) for p in sorted(set(paths))}

def build(out):
    if not (out/'inputs.json').is_file():
        raise RuntimeError('Prepare a new campaign before building')
    if any((out/name).exists() for name in ('build', 'build-provenance.json', 'capture_helpers.rs', 'occt')):
        raise FileExistsError('Build artifacts already exist; prepare a new campaign directory')
    (out/'build-stage').mkdir(exist_ok=False)
    before = sources()
    replay = (ROOT/'truck-shapeops/examples/replay_occt_blend.rs').read_text()
    # Reuse the existing capture adapter verbatim, excluding unrelated QA/replay entry points.
    helpers = replay[replay.index('use serde_json'):replay.index('fn chamfer_angle_wire(')]
    helpers += replay[replay.index('fn chamfer_distance_selection('):replay.index('fn replay(')]
    helpers += replay[replay.index('fn resolve_corner_preferences('):replay.index('fn replay_task(')]
    helper = out/'capture_helpers.rs';helper.write_text(helpers)
    env = dict(ENV, NOV3_CAPTURE_HELPERS=str(helper), CARGO_TARGET_DIR=str(out/'build'))
    run(['cargo','build','--release','--locked','-p','truck-benchmarks','--example','a6_matched'],out/'truck-build.log',env)
    libs = ['TKDESTEP','TKXSBase','TKShHealing','TKFillet','TKTopAlgo','TKGeomAlgo','TKBRep','TKGeomBase','TKG3d','TKG2d','TKMath','TKernel']
    run(['g++','-std=c++17','-O2','-isystem',SDK/'include/opencascade',ROOT/'truck-benchmarks/occt/nov3_matched.cpp','-L'+str(SDK/'lib64'),*['-l'+s for s in libs],'-o',out/'occt'],out/'occt-build.log')
    binaries=[out/'occt',out/'build/release/examples/a6_matched',ROOT/'target/occt-distance-tree-plugin/compare_shapes']
    linked = {}
    for binary in binaries:
        result=subprocess.check_output(['ldd',binary],env=ENV,text=True)
        (out/(binary.name+'.ldd')).write_text(result)
        for line in result.splitlines():
            for word in line.split():
                p=Path(word)
                if p.is_absolute() and p.is_file(): linked[str(p.resolve())]=digest(p)
    save(out/'build-provenance.json',dict(before=before,after=sources(),helpers=digest(helper),binaries={str(p):digest(p) for p in binaries},libraries=linked,static_libraries={str(p):digest(p) for p in (out/'build/release/deps').glob('*.rlib')},source_stable=before==sources()),exclusive=True)

def campaign_identity(out):
    provenance_path = out/'build-provenance.json'
    provenance = json.loads(provenance_path.read_text())
    assert provenance['source_stable'], 'Sources changed during build; use a new campaign'
    current_sources = sources()
    assert current_sources == provenance['after'], 'Source changed: rebuild and revalidate in a new campaign'
    helper = digest(out/'capture_helpers.rs')
    assert helper == provenance['helpers'], 'Capture adapter changed'
    runtime = {}
    for path, expected in (provenance['binaries'] | provenance['libraries']).items():
        runtime[path] = digest(Path(path))
        assert runtime[path] == expected, f'Binary/library changed: {path}'
    resolved_libraries = {}
    for binary in provenance['binaries']:
        linked = subprocess.check_output(['ldd', binary], env=ENV, text=True)
        for word in linked.split():
            path = Path(word)
            if path.is_absolute() and path.is_file():
                resolved_libraries[str(path.resolve())] = digest(path)
    assert resolved_libraries == provenance['libraries'], 'Runtime library resolution changed'
    inputs = {}
    for case in json.loads((out/'inputs.json').read_text()):
        for name, expected in case['hashes'].items():
            path = Path(case['folder'])/name
            inputs[str(path)] = digest(path)
            assert inputs[str(path)] == expected, f'Input/config changed: {path}'
    return dict(provenance=digest(provenance_path), sources=current_sources, helper=helper,
                runtime=runtime, input_manifest=digest(out/'inputs.json'), inputs=inputs)

def execute(out, records, smoke, samples, rounds, cpu, expected_identity=None):
    campaign = campaign_identity(out)
    assert expected_identity is None or campaign == expected_identity, 'Campaign changed after approval'
    stage = out/('smoke' if smoke else 'time')
    stage.mkdir(exist_ok=False)
    save(stage/'identity.json', campaign, exclusive=True)
    save(stage/'parameters.json', dict(cases=[c['case'] for c in records], samples=samples,
         rounds=rounds, cpu=cpu, smoke=smoke, workers=1), exclusive=True)
    results=[]
    for r in range(1 if smoke else rounds):
        for case in records:
            prefix=Path(case['folder'])/'001'
            folder=stage/case['case'].replace('/', '-')
            folder.mkdir(exist_ok=True)
            for engine in (['truck','occt'] if r%2==0 else ['occt','truck']):
                tag=folder/f'{"smoke" if smoke else "round"}-{r}-{engine}'
                mode='smoke' if smoke else str(samples)
                if engine=='truck': cmd=[out/'build/release/examples/a6_matched',prefix,mode,str(tag)+'.step']
                else: cmd=[out/'occt',str(prefix)+'.input.brep',Path(case['folder'])/'native.config',mode,tag]
                if cpu is not None: cmd=['taskset','-c',str(cpu),*cmd]
                try:
                    run(cmd,Path(str(tag)+'.log'))
                    rows=[json.loads(l) for l in Path(str(tag)+'.log').read_text().splitlines() if l.startswith('{')]
                    assert len(rows)==(3 if smoke else samples+1)
                    results.extend(dict(case=case['case'],round=r,**v) for v in rows)
                    if smoke:
                        comparison=Path(str(tag)+'.identity.json')
                        run([ROOT/'target/occt-distance-tree-plugin/compare_shapes',str(prefix)+'.result.brep',str(tag)+'.step',comparison],Path(str(tag)+'.compare.log'))
                        identity=json.loads(comparison.read_text())
                        passed=all(identity[k]<=.001 for k in ('native_to_truck_max_distance','truck_to_native_max_distance')) and identity['native_valid'] and identity['truck_step_valid'] and identity['native_solids']==identity['truck_solids']==1
                        case[engine+'_identity']=dict(passed=passed,comparison=identity)
                except (subprocess.SubprocessError,AssertionError) as e:
                    case[engine+'_blocker']=str(e)
                    if (smoke and engine == 'occt' and Path(str(tag)+'.step').exists()
                            and not Path(str(tag)+'.compare.log').exists()):
                        comparison=Path(str(tag)+'.identity.json')
                        run([ROOT/'target/occt-distance-tree-plugin/compare_shapes',str(prefix)+'.result.brep',str(tag)+'.step',comparison],Path(str(tag)+'.compare.log'))
                        identity=json.loads(comparison.read_text())
                        case[engine+'_identity']=dict(passed=False,qa_blocked=True,comparison=identity)
            save(stage/'state.json',records)
    with (stage/'records.jsonl').open('x') as f:
        for row in results:f.write(json.dumps(row)+'\n')
    assert campaign_identity(out) == campaign, 'Campaign identity changed during execution; no approval or summary'
    if smoke:
        save(stage/'approval.json', dict(identity=campaign, cases=records), exclusive=True)
    if not smoke:
        summary=[]
        for case in [c['case'] for c in records]:
            for engine in ('truck','occt'):
                values=[x['ms'] for x in results if x['case']==case and x['engine']==engine and not x['warmup']]
                assert len(values)==samples*rounds,'Missing samples; no performance verdict'
                medians=[statistics.median(x['ms'] for x in results if x['case']==case and x['engine']==engine and x['round']==r and not x['warmup']) for r in range(rounds)]
                summary.append(dict(case=case,engine=engine,median_round_median=statistics.median(medians),p95=sorted(values)[math.ceil(.95*len(values))-1],samples=len(values)))
        assert campaign_identity(out) == campaign, 'Campaign identity changed before summary publication'
        save(stage/'summary.json',summary,exclusive=True)

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('action',choices=['prepare','build','smoke','time'])
    p.add_argument('--output',type=Path,default=ROOT/'target/a6-matched')
    p.add_argument('--samples',type=int,default=10);p.add_argument('--rounds',type=int,default=4)
    p.add_argument('--case',action='append',choices=CASES,help='Repeat to select configurations (default all ten)')
    p.add_argument('--cpu',type=int);p.add_argument('--serial-slot-reserved',action='store_true')
    args=p.parse_args();out=args.output.resolve()
    if args.action=='prepare':prepare(out)
    elif args.action=='build':build(out)
    else:
        identity = campaign_identity(out)
        records=json.loads((out/'inputs.json').read_text())
        if args.case:records=[c for c in records if c['case'] in args.case]
        if args.action=='time':
            assert args.samples > 0 and args.rounds > 0
            assert args.serial_slot_reserved and args.cpu is not None,'Coordinator must reserve a serial slot and supply CPU affinity'
            approval=json.loads((out/'smoke/approval.json').read_text())
            assert approval['identity'] == identity, 'Smoke approval belongs to another build or input set'
            proof=approval['cases']
            proof=[c for c in proof if c['case'] in [r['case'] for r in records]]
            assert len(proof)==len(records),'Missing identity proofs'
            assert all(all(c.get(engine+'_identity',{}).get('passed') is True for engine in ('truck','occt')) and not any(k.endswith('_blocker') for k in c) for c in proof),'Unmatched harness; timing blocked'
        for case in records:
            for name,sha in case['hashes'].items():assert digest(Path(case['folder'])/name)==sha,'Input/config changed'
        execute(out,records,args.action=='smoke',args.samples,args.rounds,args.cpu,identity)
if __name__=='__main__': main()
