"""Replay every captured chamfer operation with the current replay binary and compare with
the exact-knot capture. Usage: python3 replay_chamfers.py OUTPUT [GLOB], with WORKERS (default 8)."""
import json, math, os, shutil, subprocess, sys
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
BASE = ROOT / 'target/occt-capture-exact-knots-2026-10-24'
REPLAY = ROOT / 'target/release/examples/replay_occt_blend'
COMPARE = ROOT / 'target/occt-distance-tree-plugin/compare_shapes'
ENV = dict(os.environ, LD_LIBRARY_PATH=str(ROOT / 'target/occt-sdk/usr/lib64'), RAYON_NUM_THREADS='1')
out = Path(sys.argv[1])
pattern = sys.argv[2] if len(sys.argv) > 2 else 'chamfer/**/capture/*.input.json'


def load(path):
    try:
        return json.loads(Path(path).read_text())
    except (OSError, ValueError):
        return None


def checks(truck, comparison):
    r = (truck or {}).get('result', {})
    m = comparison or {}
    distances = [m.get(k) for k in ['native_to_truck_max_distance', 'truck_to_native_max_distance']]
    return dict(
        constructed=r.get('status') == 'operation-succeeded',
        input_unchanged=r.get('input_unchanged') is True,
        geometry_consistent=r.get('geometry_consistent') is True,
        mesh_closed=r.get('mesh_closed') is True,
        step_prepared=r.get('step_prepared') is True,
        native_valid=m.get('native_valid') is True,
        truck_step_valid=m.get('truck_step_valid') is True,
        single_solid=m.get('native_solids') == m.get('truck_solids') == 1,
        sampled_distance=all(isinstance(d, (int, float)) and math.isfinite(d) and 0 <= d <= .001
                             for d in distances))


def run(task):
    prefix = Path(str(task).removesuffix('.input.json'))
    key = str(prefix.relative_to(BASE))
    target = out / key
    target.parent.mkdir(parents=True, exist_ok=True)
    for suffix in ['.input.json', '.input.step', '.result.brep', '.result.json']:
        source = Path(str(prefix) + suffix)
        if source.is_file():
            shutil.copy(source, str(target) + suffix)
    old_truck = load(str(prefix) + '.truck.json')
    old_cmp = load(str(prefix) + '.comparison.json')
    if old_truck is None:
        return key, None
    try:
        proc = subprocess.run([str(REPLAY), str(target)], cwd=target.parent, env=ENV,
                              capture_output=True, text=True, timeout=120)
        Path(str(target) + '.truck.log').write_text(proc.stdout + proc.stderr)
        truck = load(str(target) + '.truck.json') or dict(result=dict(status='process-failed'))
    except subprocess.TimeoutExpired:
        truck = dict(result=dict(status='timeout'))
    comparison = None
    native = load(str(target) + '.result.json')
    if truck.get('result', {}).get('step_prepared') and Path(str(target) + '.result.brep').is_file() \
            and (native or {}).get('status') != 'operation-failed':
        subprocess.run([str(COMPARE), str(target) + '.result.brep', str(target) + '.truck.step',
                        str(target) + '.comparison.json'], env=ENV, capture_output=True, timeout=300)
        comparison = load(str(target) + '.comparison.json')
    before, after = checks(old_truck, old_cmp), checks(truck, comparison)
    return key, dict(
        before=old_truck.get('result', {}).get('status'), after=truck.get('result', {}).get('status'),
        before_pass=all(before.values()), after_pass=all(after.values()),
        before_code=old_truck.get('result', {}).get('code'), after_code=truck.get('result', {}).get('code'),
        distances=[(comparison or {}).get(k) for k in ['native_to_truck_max_distance', 'truck_to_native_max_distance']],
        lost_checks=[k for k, v in before.items() if v and not after[k]])


tasks = sorted(BASE.glob(pattern))
with ThreadPoolExecutor(int(os.environ.get("WORKERS", "8"))) as pool:
    results = dict(r for r in pool.map(run, tasks) if r[1] is not None)
(out / 'summary.json').write_text(json.dumps(results, indent=1))
count = lambda f: sum(1 for r in results.values() if f(r))
print('operations', len(results))
print('constructed', count(lambda r: r['before'] == 'operation-succeeded'), '->', count(lambda r: r['after'] == 'operation-succeeded'))
print('limited passes', count(lambda r: r['before_pass']), '->', count(lambda r: r['after_pass']))
for key, r in sorted(results.items()):
    if r['before'] != r['after'] or r['before_pass'] != r['after_pass'] or r['lost_checks']:
        print(key, r['before'], '->', r['after'], 'pass', r['before_pass'], '->', r['after_pass'],
              r['after_code'] or '', r['lost_checks'] or '', r['distances'])
