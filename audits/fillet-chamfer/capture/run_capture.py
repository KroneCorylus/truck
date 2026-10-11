#!/usr/bin/env python3
"""Instrument available native blend cases and replay captured physical selections.

These instrumented executions are separate from the unchanged native audit runs.
Construction success does not establish shape equivalence or feature parity.
"""

import argparse
from concurrent.futures import ThreadPoolExecutor, as_completed
from datetime import datetime, timezone
import fnmatch
import json
from pathlib import Path
import shutil
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))
from run_occt import case_sources, classify, execute, isolated_display, sha, tcl_word


def run_case(case, sources, args, env):
    directory = args.output / case['id']
    directory.mkdir(parents=True)
    images = directory / 'images'
    images.mkdir()
    group, grid, name = case['id'].split('/')
    script = directory / 'run.tcl'
    script.write_text(
        'if {[catch {pload ALL; pload -BlendAudit BLENDAUDIT} message]} '
        '{puts stderr $message; exit 2}\n'
        f'source {tcl_word(HERE / "blend_capture.tcl")}\n'
        f'set imagedir {tcl_word(images)}\nset test_image {tcl_word(name)}\n'
        f'if {{[catch {{test {tcl_word(group)} {tcl_word(grid)} {tcl_word(name)} '
        '-outfile draw.log -errors} message]} {puts stderr $message; exit 2}\nexit\n')
    console, code, timeout, elapsed = execute(
        [args.draw, '-v', '-f', str(script)], directory, env, args.timeout)
    (directory / 'console.log').write_text(console)
    result = dict(id=case['id'], source_sha256=sources,
                  native=dict(**classify(case['id'], code, timeout, console),
                              returncode=code, elapsed_seconds=elapsed),
                  baseline_native_status=case['occt_run'], captures=[])
    for capture in sorted((directory / 'capture').glob('*.input.json')):
        prefix = str(capture).removesuffix('.input.json')
        # A failed export can leave partial JSON; only complete input/output pairs replay.
        try:
            task = json.loads(capture.read_text())
            native_result = json.loads(Path(prefix + '.result.json').read_text())
        except (ValueError, OSError) as error:
            result['captures'].append(dict(prefix=Path(prefix).name,
                                           status='capture-incomplete', error=str(error)))
            continue
        console, code, timeout, elapsed = execute(
            [str(args.replay), prefix], directory, env, args.timeout)
        Path(prefix + '.truck.log').write_text(console)
        report = Path(prefix + '.truck.json')
        if timeout:
            truck = dict(result=dict(status='timeout'))
        elif code:
            truck = dict(result=dict(status='process-failed', returncode=code))
        elif report.is_file():
            truck = json.loads(report.read_text())
        else:
            truck = dict(result=dict(status='harness-error', error='Missing replay report'))
        comparison = None
        if truck.get('result', {}).get('step_prepared'):
            log, compare_code, compare_timeout, compare_elapsed = execute(
                [str(args.plugin / 'compare_shapes'), prefix + '.result.brep',
                 prefix + '.truck.step', prefix + '.comparison.json'],
                directory, env, args.timeout)
            Path(prefix + '.comparison.log').write_text(log)
            if compare_timeout or compare_code:
                comparison = dict(status='timeout' if compare_timeout else 'process-failed',
                                  returncode=compare_code, elapsed_seconds=compare_elapsed)
            else:
                comparison = json.loads(Path(prefix + '.comparison.json').read_text())
        task['selection'] = [dict(radius=s['radius'], curve_type=s['curve_type'],
                                  point_count=len(s['points'])) for s in task['selection']]
        result['captures'].append(dict(prefix=Path(prefix).name, task=task,
                                      native_result=native_result, truck=truck,
                                      elapsed_seconds=elapsed, sampled_comparison=comparison,
                                      comparison='unverified'))
    result['artifacts'] = {str(p.relative_to(args.output)): sha(p)
                           for p in sorted(directory.rglob('*'))
                           if p.is_file() and 'images' not in p.parts}
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--occt', type=Path, required=True)
    parser.add_argument('--library-path', type=Path, required=True)
    parser.add_argument('--plugin', type=Path, required=True)
    parser.add_argument('--replay', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--case', action='append', default=[])
    parser.add_argument('--workers', type=int, default=2)
    parser.add_argument('--timeout', type=float, default=120)
    args = parser.parse_args()
    if args.workers < 1 or args.timeout <= 0:
        parser.error('workers and timeout must be positive')
    for name in ['occt', 'library_path', 'plugin', 'replay', 'output']:
        setattr(args, name, getattr(args, name).resolve())
    args.draw = shutil.which('DRAWEXE')
    if not args.draw or not args.replay.is_file() or not all(
            (args.plugin / name).is_file() for name in ['libTKBlendAudit.so', 'compare_shapes']):
        parser.error('Build DRAW plugin and replay example first; DRAWEXE must be available')
    catalog = json.loads((HERE.parent / 'catalog.json').read_text())
    cases = [c for c in catalog['cases'] if c['occt_run'] == 'passed'
             and 'blend' in c['commands'] and (not args.case or
             any(fnmatch.fnmatchcase(c['id'], pattern) for pattern in args.case))]
    if not cases:
        parser.error('No passing native blend cases selected')
    sources = {c['id']: case_sources(c, args.occt) for c in cases}
    args.output.mkdir(parents=True, exist_ok=False)
    manifest = dict(schema_version=1, started_utc=datetime.now(timezone.utc).isoformat(),
                    scope=__doc__, snapshots=catalog['snapshots'],
                    source_sha256={str(p): sha(p) for p in [Path(__file__),
                        HERE / 'blend_capture.cpp', HERE / 'blend_capture.tcl', HERE / 'compare_shapes.cpp',
                        HERE.parent / 'run_occt.py',
                        HERE.parents[2] / 'truck-shapeops/examples/replay_occt_blend.rs']},
                    executable_sha256={str(p): sha(p) for p in [Path(args.draw), args.replay,
                        args.plugin / 'libTKBlendAudit.so', args.plugin / 'compare_shapes']},
                    library_sha256={p.name: sha(p) for p in sorted(args.library_path.glob('*.so')) if p.is_file()},
                    harness_sha256={str(p.relative_to(args.occt)): sha(p)
                                    for p in sorted((args.occt / 'src/DrawResources').glob('*')) if p.is_file()},
                    timeout_seconds=args.timeout, workers=args.workers, cases=[])
    with isolated_display(args.output) as env:
        env.update(LD_LIBRARY_PATH=f'{args.plugin}:{args.library_path}', CASROOT=str(args.occt),
                   CSF_BlendAuditDefaults=str(args.plugin),
                   CSF_DrawPluginDefaults=str(args.occt / 'src/DrawResources'),
                   DRAWDEFAULT=str(args.occt / 'src/DrawResources/DrawDefault'),
                   CSF_TestScriptsPath=str(args.occt / 'tests'),
                   CSF_TestDataPath=str(args.occt / 'data'),
                   CSF_ShadersDirectory=str(args.occt / 'src/Shaders'))
        version, code, timed_out, _ = execute([args.draw, '-v', '-c',
            'pload ALL; pload -BlendAudit BLENDAUDIT; puts [dversion]; exit'], args.output, env, 30)
        (args.output / 'runtime.log').write_text(version)
        if code or timed_out or 'Open CASCADE Technology' not in version:
            raise RuntimeError('Runtime preflight failed; see runtime.log')
        manifest['runtime'] = version
        with ThreadPoolExecutor(max_workers=args.workers) as pool:
            tasks = [pool.submit(run_case, c, sources[c['id']], args, env) for c in cases]
            for task in as_completed(tasks):
                result = task.result()
                manifest['cases'].append(result)
                manifest['cases'].sort(key=lambda c: c['id'])
                temporary = args.output / 'results.tmp'
                temporary.write_text(json.dumps(manifest, indent=2) + '\n')
                temporary.replace(args.output / 'results.json')
                outcomes = [c.get('truck', {}).get('result', {}).get('status', c.get('status'))
                            for c in result['captures']]
                print(f'{len(manifest["cases"])}/{len(cases)} {result["id"]}: '
                      f'{result["native"]["status"]} {outcomes}', flush=True)


if __name__ == '__main__':
    main()
