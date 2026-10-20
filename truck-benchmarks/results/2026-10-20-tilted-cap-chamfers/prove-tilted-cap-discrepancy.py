from pathlib import Path
import json,math,hashlib
rows=[]
for line in Path('target/tilted-cap-boundary.txt').read_text().splitlines():
 if line.startswith('MAX '):break
 if line.startswith('EDGE26 '):rows.append(tuple(map(float,line[7:].split(','))))
assert len(rows)==25
h=.782539615254126;k=math.tan(math.radians(10));theta=-1.148603555445661
witness=(.6+.4*math.cos(theta),.4+.4*math.sin(theta),h-k*(.6+.4*math.cos(theta)))
expected=(.72553121980511337,.12320413054493012,.65460888585616073)
p=min(rows,key=lambda p:math.dist(p,expected));assert math.dist(p,expected)<1e-14
assert abs(p[2]-(h-k*p[0]))<1e-12
assert -math.pi/2<theta<0 and abs(math.hypot(witness[0]-.6,witness[1]-.4)-.4)<1e-14
upper_bound=math.dist(p,witness);assert upper_bound<.097
records=json.load(open('target/tilted-cap-comparison-summary.json'));q=next(q for q in records if q['mode']=='angle10')
assert q['max_setback_error']<3e-6
assert q['result_comparison']['native_to_truck_max_distance']>.0025
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
proof=dict(native_cap_boundary_point=p,original_guide_witness=witness,witness_parameter=theta,nearest_guide_distance_upper_bound=upper_bound,requested_setback=.1,minimum_setback_shortfall=.1-upper_bound,truck_contact_sample_max_error=q['max_setback_error'],native_to_truck_distance=q['result_comparison']['native_to_truck_max_distance'],classification='Unresolved native reference discrepancy; existing distance comparison fails. This witness proves the native cap contact is not a constant 0.1 distance from the original contour. It does not establish the cause or global correctness of Truck surfaces.',evidence_sha256={p:sha(p) for p in ['target/tilted-cap-boundary.txt','target/tilted-cap-boundary.cpp','target/compare-tilted-cap.py','target/tilted-cap-native-probe.txt','target/tilted-cap-comparison-summary.json']})
Path('target/tilted-cap-discrepancy-proof.json').write_text(json.dumps(proof,indent=2)+'\n');print(json.dumps(proof,indent=2))
