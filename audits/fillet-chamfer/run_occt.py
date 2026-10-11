#!/usr/bin/env python3
"""Execute inventoried OCCT cases unchanged on an isolated headless display.

Requires DRAWEXE, sway and Xwayland. Default selection excludes cases with known
missing static fixtures. Results describe OCCT's harness, never Truck parity.
"""

import argparse
from concurrent.futures import ThreadPoolExecutor, as_completed
from contextlib import contextmanager
from datetime import datetime, timezone
import fnmatch
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import signal
import subprocess
import tempfile
import time

HERE = Path(__file__).resolve().parent
VERDICTS = {'OK': 'passed', 'BAD': 'known-failure', 'FAILED': 'failed',
            'IMPROVEMENT': 'improvement', 'SKIPPED': 'skipped'}


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def tcl_word(value):
    return '"' + re.sub(r'([\\"$\[\]])', r'\\\1', str(value)).replace('\n', '\\n').replace('\r', '\\r') + '"'


def classify(case_id, returncode, timed_out, console):
    group, grid, case = case_id.split('/')
    pattern = rf'^CASE\s+{re.escape(group)}\s+{re.escape(grid)}\s+{re.escape(case)}:\s+(\w+)(.*)$'
    summaries = re.findall(pattern, console, re.M)
    verdict, detail = summaries[-1] if summaries else (None, '')
    if timed_out:
        status = 'timeout'
    elif returncode < 0:
        status = 'crash'
    elif returncode != 0 or verdict not in VERDICTS:
        status = 'harness-error'
    else:
        status = VERDICTS[verdict]
    return dict(status=status, harness_verdict=verdict, harness_detail=detail.strip())


def execute(command, cwd, env, timeout):
    start = time.monotonic()
    with subprocess.Popen(command, cwd=cwd, env=env, stdout=subprocess.PIPE,
                          stderr=subprocess.STDOUT, start_new_session=True) as process:
        timed_out = False
        try:
            console, _ = process.communicate(timeout=timeout)
        except subprocess.TimeoutExpired:
            timed_out = True
            os.killpg(process.pid, signal.SIGKILL)
            console, _ = process.communicate()
    return console.decode('utf-8', errors='replace'), process.returncode, timed_out, time.monotonic() - start


@contextmanager
def isolated_display(output):
    # Start a separate compositor; never inherit the user's display or control socket.
    env = os.environ.copy()
    for key in ['DISPLAY', 'WAYLAND_DISPLAY', 'SWAYSOCK']:
        env.pop(key, None)
    env.update(WLR_BACKENDS='headless', WLR_RENDERER='pixman', WLR_LIBINPUT_NO_DEVICES='1')
    with tempfile.TemporaryDirectory(prefix='occt-display-') as temp:
        temp = Path(temp)
        display = temp / 'display'
        config = temp / 'config'
        config.write_text('output HEADLESS-1 mode 1280x800\nxwayland force\n'
                          f'exec printf "%s\\n" "$DISPLAY" > {display}\n')
        with (output / 'display.log').open('w') as log:
            process = subprocess.Popen(['sway', '-c', str(config)], env=env,
                                       stdout=log, stderr=subprocess.STDOUT)
            try:
                for _ in range(100):
                    if display.exists() and display.read_text().strip():
                        env['DISPLAY'] = display.read_text().strip()
                        yield env
                        return
                    if process.poll() is not None:
                        break
                    time.sleep(0.1)
                raise RuntimeError(f'Isolated display did not start; see {output / "display.log"}')
            finally:
                if process.poll() is None:
                    process.terminate()
                    try:
                        process.wait(timeout=10)
                    except subprocess.TimeoutExpired:
                        process.kill()
                        process.wait()


def case_sources(case, occt):
    sources = {case['source']['path']: case['sha256']}
    sources.update((h['path'], h['sha256']) for h in case['helpers'])
    for path, expected in sources.items():
        if sha(occt / path) != expected:
            raise ValueError(f'Stale catalog source: {path}')
    # The harness also evaluates group/grid parse rules, which affect its verdict.
    for prefix in ['tests', f'tests/{case["group"]}', f'tests/{case["group"]}/{case["grid"]}']:
        path = f'{prefix}/parse.rules'
        if (occt / path).is_file():
            sources[path] = sha(occt / path)
    return sources


def run_case(case, sources, args, env):
    directory = args.output / case['id']
    directory.mkdir(parents=True)
    images = directory / 'images'
    images.mkdir()
    script = directory / 'run.tcl'
    group, grid, name = case['id'].split('/')
    script.write_text(
        'if {[catch {pload ALL} message]} {puts stderr $message; exit 2}\n'
        f'set imagedir {tcl_word(images)}\nset test_image {tcl_word(name)}\n'
        f'if {{[catch {{test {tcl_word(group)} {tcl_word(grid)} {tcl_word(name)} '
        '-outfile draw.log -errors} message]} {puts stderr $message; exit 2}\nexit\n')
    console, code, timeout, elapsed = execute([args.draw, '-v', '-f', str(script)], directory, env, args.timeout)
    (directory / 'console.log').write_text(console)
    logs = {}
    for filename in ['console.log', 'draw.log', 'run.tcl']:
        path = directory / filename
        if path.exists():
            logs[str(path.relative_to(args.output))] = sha(path)
    return dict(id=case['id'], **classify(case['id'], code, timeout, console),
                returncode=code, elapsed_seconds=round(elapsed, 6), timeout_seconds=args.timeout,
                source_sha256=sources, artifacts=logs, truck_run='not-run', comparison='unverified')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--occt', type=Path, required=True)
    parser.add_argument('--library-path', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True, help='New directory; existing results are never overwritten')
    parser.add_argument('--catalog', type=Path, default=HERE / 'catalog.json')
    parser.add_argument('--case', action='append', default=[], help='Case ID glob, repeatable')
    parser.add_argument('--include-missing', action='store_true')
    parser.add_argument('--draw', default='DRAWEXE')
    parser.add_argument('--workers', type=int, default=2)
    parser.add_argument('--timeout', type=float, default=120)
    args = parser.parse_args()
    if args.workers < 1 or args.timeout <= 0:
        parser.error('workers and timeout must be positive')
    args.occt, args.output, args.library_path = args.occt.resolve(), args.output.resolve(), args.library_path.resolve()
    args.draw = shutil.which(args.draw)
    if not args.draw:
        parser.error('DRAWEXE was not found')
    catalog = json.loads(args.catalog.read_text())
    cases = [c for c in catalog['cases'] if
             (args.include_missing or c['dependency_status'] != 'missing-in-checkout') and
             (not args.case or any(fnmatch.fnmatchcase(c['id'], p) for p in args.case))]
    if not cases:
        parser.error('No cases selected')
    sources = {case['id']: case_sources(case, args.occt) for case in cases}
    args.output.mkdir(parents=True, exist_ok=False)
    manifest = dict(schema_version=1, started_utc=datetime.now(timezone.utc).isoformat(),
                    scope='Unmodified OCCT harness execution; no Truck comparison or proof of blend geometry.',
                    source_revision=catalog['snapshots']['occt']['revision'],
                    runner_sha256=sha(__file__), workers=args.workers,
                    draw=dict(path=args.draw, sha256=sha(args.draw)),
                    harness_sha256={str(p.relative_to(args.occt)): sha(p)
                                    for p in sorted((args.occt / 'src/DrawResources').glob('*')) if p.is_file()},
                    library_sha256={p.name: sha(p) for p in sorted(args.library_path.glob('*.so')) if p.is_file()},
                    cases=[])
    with isolated_display(args.output) as env:
        env.update(LD_LIBRARY_PATH=str(args.library_path), CASROOT=str(args.occt),
                   CSF_DrawPluginDefaults=str(args.occt / 'src/DrawResources'),
                   DRAWDEFAULT=str(args.occt / 'src/DrawResources/DrawDefault'),
                   CSF_TestScriptsPath=str(args.occt / 'tests'), CSF_TestDataPath=str(args.occt / 'data'),
                   CSF_ShadersDirectory=str(args.occt / 'src/Shaders'))
        version, code, timed_out, _ = execute([args.draw, '-v', '-c', 'pload ALL; puts [dversion]; exit'],
                                              args.output, env, 30)
        (args.output / 'runtime.log').write_text(version)
        if code or timed_out or 'Open CASCADE Technology' not in version:
            raise RuntimeError('Runtime preflight failed; see runtime.log')
        manifest['runtime'] = version
        with ThreadPoolExecutor(max_workers=args.workers) as pool:
            tasks = [pool.submit(run_case, c, sources[c['id']], args, env) for c in cases]
            for task in as_completed(tasks):
                result = task.result()
                manifest['cases'].append(result)
                manifest['cases'].sort(key=lambda r: r['id'])
                temporary = args.output / 'results.tmp'
                temporary.write_text(json.dumps(manifest, indent=2) + '\n')
                temporary.replace(args.output / 'results.json')
                print(f'{len(manifest["cases"])}/{len(cases)} {result["id"]}: {result["status"]}', flush=True)


if __name__ == '__main__':
    main()
