from pathlib import Path
import math,json,hashlib
summary=json.load(open('target/tilted-cap-unequal-comparison-summary.json'));proofs=[];sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
for row in summary['controls']:
 if row['limited_distance_pass']:continue
 q=row['native'];path=Path('target/tilted-cap-unequal-discrepancies')/(q['mode']+'.txt');line=path.read_text().splitlines()[0];assert line.startswith('MAX edge 26 ')
 p=tuple(map(float,line.split(' point ',1)[1].split(' closest ',1)[0].split(',')));k=math.tan(math.radians(q['degrees']));h=.782539615254126;assert abs(p[2]-(h-k*p[0]))<1e-10
 def guide(t):
  x=.6+.4*math.cos(t);return (x,.4+.4*math.sin(t),h-k*x)
 def distance(t):return math.dist(p,guide(t))
 a,b=-math.pi/2,0.;ratio=(math.sqrt(5)-1)/2
 for _ in range(120):
  c=b-ratio*(b-a);d=a+ratio*(b-a)
  if distance(c)<distance(d):b=d
  else:a=c
 t=(a+b)/2;witness=guide(t);bound=math.dist(p,witness);assert -math.pi/2<t<0
 assert abs(math.hypot(witness[0]-.6,witness[1]-.4)-.4)<1e-14
 shortfall=q['cap_distance']-bound;assert shortfall>.001,(q['mode'],bound)
 proofs.append(dict(mode=q['mode'],native_cap_boundary_point=p,original_guide_witness=witness,witness_parameter=t,requested_cap_setback=q['cap_distance'],nearest_guide_distance_upper_bound=bound,minimum_setback_shortfall=shortfall,sampled_distance_failure=max(row['comparison']['native_to_truck_max_distance'],row['comparison']['truck_to_native_max_distance']),extraction_log=str(path),extraction_log_sha256=sha(path)))
assert len(proofs)==3
result=dict(classification='Three retained native distance-comparison failures. Explicit witnesses on the original guide prove native cap-contact shortfalls larger than0.001. This is not a global Truck accuracy proof; native smoothing is a possible but unproven cause.',proofs=proofs,comparison_sha256=sha('target/tilted-cap-unequal-comparison-summary.json'))
Path('target/tilted-cap-unequal-discrepancy-proof.json').write_text(json.dumps(result,indent=2)+'\n')
for q in proofs:print(q['mode'],'native setback <=',q['nearest_guide_distance_upper_bound'],'requested',q['requested_cap_setback'],'shortfall >=',q['minimum_setback_shortfall'])
