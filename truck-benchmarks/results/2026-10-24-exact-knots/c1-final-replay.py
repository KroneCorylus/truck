#!/usr/bin/env python3
"""Serial final-binary replay of the 24 original-scale C1 sequence captures.
Does not build, alter inputs, or update the canonical corpus. Output must be new.
"""
import argparse
from collections import Counter
from datetime import datetime, timezone
import hashlib
import json
import math
import os
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'audits/fillet-chamfer'))
from run_occt import execute

GRIDS = ('dist_angle_sequence', 'dist_dist_sequence', 'equal_dist_sequence')
NATIVE_SUFFIXES = ('input.brep', 'input.json', 'input.step', 'result.brep', 'result.json', 'result.step')

def sha(path):
    h = hashlib.sha256()
    with Path(path).open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            h.update(block)
    return h.hexdigest()

def git(*args):
    return subprocess.check_output(['git', '-C', str(ROOT), *args], text=True).strip()

def sources():
    names = git('ls-files', '--cached', '--others', '--exclude-standard', '--',
                '*.rs', '*/Cargo.toml', 'Cargo.toml', 'Cargo.lock').splitlines()
    return {name: sha(ROOT / name) for name in sorted(set(names)) if (ROOT / name).is_file()}

def load(path):
    return json.loads(Path(path).read_text())

def status(record):
    return (record.get('truck') or {}).get('result', {}).get('status', record.get('status'))

def checks(record):
    result = (record.get('truck') or {}).get('result', {})
    comparison = record.get('sampled_comparison') or {}
    distances = [comparison.get(k) for k in ('native_to_truck_max_distance', 'truck_to_native_max_distance')]
    return dict(constructed=status(record) == 'operation-succeeded',
                **{k: result.get(k) is True for k in ('input_unchanged', 'geometry_consistent', 'mesh_closed', 'step_prepared')},
                native_valid=comparison.get('native_valid') is True,
                truck_step_valid=comparison.get('truck_step_valid') is True,
                single_solid=comparison.get('native_solids') == comparison.get('truck_solids') == 1,
                sampled_distance=all(isinstance(d, (int, float)) and math.isfinite(d) and 0 <= d <= .001 for d in distances))

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True, help='Fresh directory; existing directories are rejected')
    parser.add_argument('--baseline', type=Path, default=ROOT / 'target/occt-capture-mixed-curved-corners')
    parser.add_argument('--replay', type=Path, default=ROOT / 'target/release/examples/replay_occt_blend')
    parser.add_argument('--comparator', type=Path, default=ROOT / 'target/occt-distance-tree-plugin/compare_shapes')
    parser.add_argument('--library-path', type=Path, default=ROOT / 'target/occt-sdk/usr/lib64')
    parser.add_argument('--timeout', type=float, default=120., help='Per replay/comparator process, canonical capture default120seconds')
    args = parser.parse_args()
    if args.timeout <= 0 or not math.isfinite(args.timeout):
        parser.error('timeout must be finite and positive')
    for name in ('output', 'baseline', 'replay', 'comparator', 'library_path'):
        setattr(args, name, getattr(args, name).resolve())
    for path in (args.replay, args.comparator, args.baseline / 'results.json'):
        if not path.is_file():
            parser.error(f'Missing required file: {path}')
    if not args.library_path.is_dir():
        parser.error(f'Missing SDK library directory: {args.library_path}')
    if args.output.exists():
        parser.error('output directory already exists; refusing stale outputs')
    baseline_manifest = load(args.baseline / 'results.json')
    previous = {(c['id'], q['prefix']): q for c in baseline_manifest['cases'] for q in c['captures']}
    cases = [(f'chamfer/{grid}/C1', f'{i:03}') for grid in GRIDS for i in range(1, 9)]
    native_hashes = {path: digest for c in baseline_manifest['cases'] for path, digest in c.get('artifacts', {}).items()}
    for case, prefix in cases:
        if (case, prefix) not in previous:
            parser.error(f'Missing baseline manifest key {case}/{prefix}')
        for suffix in NATIVE_SUFFIXES:
            path = args.baseline / case / 'capture' / f'{prefix}.{suffix}'
            if not path.is_file():
                parser.error(f'Missing native artifact {path}')
            relative = str(path.relative_to(args.baseline))
            if native_hashes.get(relative) != sha(path):
                parser.error(f'Baseline artifact missing recorded hash or changed: {path}')
    args.output.mkdir(parents=True)
    env = os.environ.copy()
    env['LD_LIBRARY_PATH'] = os.pathsep.join(filter(None, (str(args.comparator.parent), str(args.library_path), env.get('LD_LIBRARY_PATH'))))
    # Comparator and replay are headless; no display/compositor or DRAW runtime is needed.
    executable_hashes = {str(path): sha(path) for path in (args.replay, args.comparator)}
    source_hashes = sources()
    manifest = dict(schema_version=1, started=datetime.now(timezone.utc).isoformat(),
                    root=str(ROOT), baseline=str(args.baseline), output=str(args.output),
                    git_head=git('rev-parse', 'HEAD'), git_status=git('status', '--porcelain'),
                    kernel_source_sha256=source_hashes, executable_sha256=executable_hashes,
                    runner_sha256=sha(__file__), baseline_manifest_sha256=sha(args.baseline / 'results.json'),
                    timeout_seconds=args.timeout, workers=1, original_scale=True,
                    environment={k: env.get(k) for k in ('LD_LIBRARY_PATH', 'RAYON_NUM_THREADS')},
                    comparison_rule='Canonical limited-pass conjunction; bidirectional sampled distance <=0.001. No volume threshold added. Not a global equivalence proof.',
                    captures=[], complete=False)
    def flush():
        temporary = args.output / 'results.tmp'
        temporary.write_text(json.dumps(manifest, indent=2) + '\n')
        temporary.replace(args.output / 'results.json')
    flush()
    copied = {}
    for number, (case, name) in enumerate(cases, 1):
        directory = args.output / case / 'capture'
        directory.mkdir(parents=True, exist_ok=True)
        prefix = directory / name
        for suffix in NATIVE_SUFFIXES:
            source = args.baseline / case / 'capture' / f'{name}.{suffix}'
            destination = Path(f'{prefix}.{suffix}')
            digest = sha(source)
            shutil.copy2(source, destination)
            if sha(destination) != digest:
                raise RuntimeError(f'Copy hash mismatch: {destination}')
            copied[str(destination.relative_to(args.output))] = dict(source=str(source), sha256=digest)
        before = previous[(case, name)]
        Path(f'{prefix}.baseline.json').write_text(json.dumps(before, indent=2) + '\n')
        command = [str(args.replay), str(prefix)]
        log, code, timed_out, elapsed = execute(command, directory, env, args.timeout)
        Path(f'{prefix}.truck.log').write_text(log)
        report = Path(f'{prefix}.truck.json')
        if timed_out:
            truck = dict(result=dict(status='timeout'))
        elif code:
            truck = dict(result=dict(status='process-failed', returncode=code))
        elif report.is_file():
            truck = load(report)
        else:
            truck = dict(result=dict(status='harness-error', error='Missing replay report'))
        comparison = None
        comparison_process = None
        if Path(f'{prefix}.truck.step').is_file():
            compare_command = [str(args.comparator), f'{prefix}.result.brep', f'{prefix}.truck.step', f'{prefix}.comparison.json']
            log, compare_code, compare_timeout, compare_elapsed = execute(compare_command, directory, env, args.timeout)
            Path(f'{prefix}.comparison.log').write_text(log)
            comparison_process = dict(command=compare_command, returncode=compare_code, timeout=compare_timeout, elapsed_seconds=compare_elapsed)
            if compare_timeout or compare_code:
                comparison = dict(status='timeout' if compare_timeout else 'process-failed', returncode=compare_code, elapsed_seconds=compare_elapsed)
            elif Path(f'{prefix}.comparison.json').is_file():
                comparison = load(f'{prefix}.comparison.json')
            else:
                comparison = dict(status='harness-error', error='Missing comparison report')
        after = dict(case=case, prefix=name, task=load(f'{prefix}.input.json'), native_result=load(f'{prefix}.result.json'),
                     truck=truck, elapsed_seconds=elapsed, sampled_comparison=comparison, comparison='unverified',
                     replay_process=dict(command=command, returncode=code, timeout=timed_out), comparison_process=comparison_process)
        before_checks, after_checks = checks(before), checks(after)
        after['before_after'] = dict(before_status=status(before), after_status=status(after), before_checks=before_checks, after_checks=after_checks,
                                     before_limited_pass=all(before_checks.values()), after_limited_pass=all(after_checks.values()),
                                     before_elapsed_seconds=before.get('elapsed_seconds'), after_elapsed_seconds=elapsed)
        manifest['captures'].append(after)
        manifest['copied_native_artifacts'] = copied
        flush()
        print(f'{number}/24 {case}/{name}: {status(before)} -> {status(after)}; limited={all(after_checks.values())}; {elapsed:.3f}s', flush=True)
    source_after = sources()
    binaries_after = {path: sha(path) for path in executable_hashes}
    manifest['sources_unchanged'] = source_after == source_hashes
    manifest['binaries_unchanged'] = binaries_after == executable_hashes
    manifest['native_artifacts_unchanged'] = all(sha(args.output / relative) == info['sha256'] == sha(info['source']) for relative, info in copied.items())
    manifest['baseline_manifest_unchanged'] = sha(args.baseline / 'results.json') == manifest['baseline_manifest_sha256']
    manifest['artifacts_sha256'] = {str(p.relative_to(args.output)): sha(p) for p in sorted(args.output.rglob('*')) if p.is_file() and p.name not in ('results.json', 'results.tmp')}
    manifest['summary'] = dict(before_statuses=dict(Counter(q['before_after']['before_status'] for q in manifest['captures'])),
                               after_statuses=dict(Counter(status(q) for q in manifest['captures'])),
                               before_limited_passes=sum(q['before_after']['before_limited_pass'] for q in manifest['captures']),
                               after_limited_passes=sum(q['before_after']['after_limited_pass'] for q in manifest['captures']))
    verified = all(manifest[k] for k in ('sources_unchanged', 'binaries_unchanged', 'native_artifacts_unchanged', 'baseline_manifest_unchanged'))
    manifest['complete'] = verified and len(manifest['captures']) == 24
    manifest['finished'] = datetime.now(timezone.utc).isoformat()
    flush()
    if not verified:
        raise RuntimeError('Source/binary/native artifact drift detected; results not accepted as frozen final-production evidence')
    print(json.dumps(manifest['summary'], indent=2))

if __name__ == '__main__':
    main()
