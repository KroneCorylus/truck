"""Summarize retained H1 witnesses; no global bounds or native parity claims."""
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

OUT = Path('target/h1-local-witness-20261103')

def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()

if '--snapshot' in sys.argv:
    files = subprocess.check_output(['git','ls-files','-z']).decode().split('\0')
    files += ['truck-shapeops/tests/cubic_remote_junction.rs', 'audits/fillet-chamfer/oracles/h1_material.py']
    (OUT / 'source-before.json').write_text(json.dumps({p:digest(p) for p in files if p and Path(p).is_file()}, indent=2))
    raise SystemExit(0)

text = (OUT / 'localized.log').read_text()
base = 'target/h1-implementation-20261103/captures/H1/001'
with (OUT / 'edges.log').open('w') as output:
    for line in text.splitlines():
        if ' edge_refined distance ' not in line:
            continue
        match = re.search(r'source_edge (\d+) parameters (\S+)', line)
        subprocess.run([str(OUT / 'h1_edge_witness'), base+'.result.brep', base+'.truck.step',
            'truck' if line.startswith('truck') else 'native', match[1], match[2]], stdout=output, check=True)
regions = {}
for line in text.splitlines():
    if not line.startswith('witness '):
        continue
    name = line.split()[1]
    row = regions.setdefault(name, dict(samples=0, truck_material_failures=0,
        native_material_failures=0, max_truck_boundary_distance=0., max_native_boundary_distance=0.))
    row['samples'] += 1
    states = re.findall(r'offset .*? native (\w+) truck (\w+)', line)
    assert len(states) == 2, line
    row['truck_material_failures'] += [v[1] for v in states] != ['IN', 'OUT']
    row['native_material_failures'] += [v[0] for v in states] != ['IN', 'OUT']
    for solid in ['truck', 'native']:
        key = 'max_' + solid + '_boundary_distance'
        row[key] = max(row[key], float(re.search(solid + r'_distance (\S+)', line)[1]))
assert set(regions) == {'P-Q','Q-R','canal','sphere','sphere-ruling-junction',
    'ruling-cylinder','bottom-edge','bottom-interior','original-left-ruling'}
assert all(r['truck_material_failures'] == 0 for r in regions.values())
assert all(r['max_truck_boundary_distance'] < 1.1764851298470662e-5 for r in regions.values())
assert regions['sphere']['native_material_failures'] > 0
before = json.loads((OUT / 'source-before.json').read_text())
changes = [p for p, h in before.items() if Path(p).exists() and digest(p) != h]
production_changes = [p for p in changes if '/src/' in p or '/tests/' in p]
assert not production_changes, production_changes
files = list(Path('target/h1-implementation-20261103/captures/H1').glob('001.*'))
files += [Path('target/h1-implementation-20261103/build/release/examples/replay_occt_blend'),
    Path('target/h1-implementation-20261103/source-provenance.json'),
    Path('target/h1-implementation-20261103/validation-results.json'),
    Path('target/h1-implementation-20261103/parent-proof.json'),
    OUT / 'h1_witness', OUT / 'h1_edge_witness', OUT / 'edges.log', OUT / 'localized.log']
files += list(Path('audits/fillet-chamfer/oracles').glob('h1_*'))
files += [Path(p) for p in re.findall(r'=> (/\S+)', (OUT / 'libraries.txt').read_text())]
identities = {str(p): digest(p) for p in files if p.is_file()}
(OUT / 'identities.json').write_text(json.dumps(identities, indent=2) + '\n')
provenance = json.loads(Path('target/h1-implementation-20261103/source-provenance.json').read_text())
replay_path = 'target/h1-implementation-20261103/build/release/examples/replay_occt_blend'
result = dict(
    replay_binary=dict(retained_expected=provenance['replay_binary_sha256'], present=digest(replay_path),
        matches_retained_provenance=digest(replay_path)==provenance['replay_binary_sha256']),
    classification='local rolling-ball contract mismatch at sampled right sphere and canal; no demonstrated Truck defect',
    source_scale=1, capture_scale=10, source_radius=.5, capture_radius=5,
    equations=dict(B='(6t+4t^3,30t-42t^2+12t^3,0)', A=[0,2,10],
        left_phi0_root=.0212268503752104, right_phi0_root=.950838009792891,
        left_phibeta_root=.000846756199312028, right_phibeta_root=.997943594311940,
        right_root_note='Additional right roots are not evidence of a wrong branch.'),
    log_encoding=dict(surface_types={0:'plane',1:'cylinder',3:'sphere',6:'BSpline',8:'extrusion'},
        orientations={0:'forward',1:'reversed'}, support_types={0:'vertex',1:'edge',2:'face'},
        parameters='Face coordinates are UV; edge coordinate is t (edges have no intrinsic surface UV). Target face UV includes faces within 1e-6 of the target witness.'),
    source_volume=372, independent_candidate_flux=367.45551955681,
    native_is_not_volume_oracle=True,
    grid=24, local_refinement_rounds=5, refinement_factor=4,
    material_offset=.01, localized_canal_offset=.0005,
    classifier_tolerance=1e-7, trimmed_face_tolerance=1e-8,
    regions=regions,
    directional_maxima=[line for line in text.splitlines() if ' refined distance ' in line or ' edge_refined distance ' in line],
    equation_evidence=[line for line in text.splitlines() if 'canal_equation' in line or 'sphere_equation' in line or 'maximum_sphere' in line],
    tolerances=[line for line in text.splitlines() if 'tolerance_kind' in line],
    supplemental_edge_evidence=(OUT / 'edges.log').read_text(),
    identity_file='identities.json', source_preservation=dict(production_test_changes=production_changes, other_tracked_changes=changes),
    provenance_limits='Queries link the recorded /lib64 OCCT libraries; original native capture producer binary identity is not established by these library hashes. Present replay binary differs from retained producer hash; the original replay producer binary is not recovered. Exact retained STEP/BREP artifacts are hashed and queried without regeneration.',
    unresolved=['Truck sphere-canal edge 13 reports distance .000963109694 to native, but both solids classify independent radial offsets +/- .0003 correctly. This seam distance/query-versus-material behavior remains unresolved; no material defect inferred there.',
        'No global Hausdorff bound; local grid/refinement may miss other maxima.',
        'No global containment, root-branch completeness, uniqueness or nonintersection proof.',
        'Floating-point calculations are not outward-rounded interval certificates.',
        'No native construction trace; BSpline surface type does not identify filling algorithm.',
        'Broader fitted-face containment, profile perturbation and incident/opposite-use topology coverage remain review follow-ups.',
        'No production fix warranted by these local witnesses; native comparison remains failed.'])
(OUT / 'results.json').write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps(dict(classification=result['classification'], regions=regions, production_test_changes=production_changes),indent=2))
