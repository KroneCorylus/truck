#!/usr/bin/env python3
"""C2 equal/DD/DA plus reviewed G4/A5 neighbors; reuse the matched adapters.

All stages require an explicit fresh campaign --output. Build/smoke need a
separately assigned slot. Material approval must hash every fresh smoke artifact,
as well as independent oracle inputs, tools and results. No historical approval
authorizes timing. See g4_a5_matched for the approval format and stage commands.
"""
import importlib.util
import json
from pathlib import Path
import sys


spec = importlib.util.spec_from_file_location(
    'c2_shared_matched', Path(__file__).with_name('g4_a5_matched.py'))
harness = importlib.util.module_from_spec(spec)
spec.loader.exec_module(harness)
harness.CASES = [f'chamfer/{mode}_complex/C2' for mode in
                 ('equal_dist', 'dist_dist', 'dist_angle')] + harness.CASES
original_sources = harness.sources
original_material_identity = harness.material_identity


def sources():
    result = original_sources()
    for name in ('c2_matched.py', 'test_c2_matched.py'):
        path = Path(__file__).with_name(name)
        result[str(path.relative_to(harness.ROOT))] = harness.digest(path)
    return result


def material_identity(path, out, records, campaign):
    result = original_material_identity(path, out, records, campaign)
    approval = json.loads(path.read_text())
    smoke = json.loads((out/'smoke/approval.json').read_text())
    assert smoke['identity'] == campaign, 'Stale smoke identity'
    rows = {}
    for row in smoke['cases']:
        assert row['case'] not in rows, 'Duplicate smoke case'
        rows[row['case']] = row
    assert records, 'Empty campaign'
    for record in records:
        row = rows[record['case']]
        assert all(row[k] == record[k] for k in ('case', 'folder', 'hashes')), 'Smoke input mismatch'
        assert not any(k.endswith('_blocker') for k in row), 'Failed smoke'
        assert all(row.get(engine+'_identity', {}).get('passed') is True
                   for engine in ('truck', 'occt')), 'Missing or failed smoke engine'
    # Include approval, raw logs and comparisons, not just the exported STEP.
    files = [p for p in (out/'smoke').rglob('*') if p.is_file()]
    for evidence in files:
        assert approval['proof'].get(str(evidence.resolve())) == harness.digest(evidence), \
            f'Material proof omits or changed smoke evidence: {evidence}'
    result['smoke'] = {str(p.resolve()): harness.digest(p) for p in files}
    return result


harness.sources = sources
harness.material_identity = material_identity


def main():
    if not any(a == '--output' or a.startswith('--output=') for a in sys.argv[1:]):
        raise SystemExit('C2 requires explicit --output for every stage')
    if sys.flags.optimize:
        raise SystemExit('Integrity assertions require Python without -O')
    harness.main()


if __name__ == '__main__':
    main()
