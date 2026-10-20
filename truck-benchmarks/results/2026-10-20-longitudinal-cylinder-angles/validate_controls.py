from pathlib import Path
import json,subprocess,os,hashlib,math
root=Path.cwd();proof=root/'truck-benchmarks/results/2026-10-20-longitudinal-cylinder-angles';raw=root/'target/native-longitudinal-cylinder-angles'
v=json.loads((proof/'validation.json').read_text());sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
for p,hsh in v['source_sha256'].items():assert sha(p)==hsh,p
replay=root/'target/release/examples/replay_occt_blend';assert sha(replay)==v['replay_sha256']
plugin=root/'target/occt-cone-reference-plugin';env=dict(os.environ,LD_LIBRARY_PATH=str(root/'target/occt-sdk/usr/lib64'))
records=json.loads((raw/'summary.json').read_text())
for q in records:
 prefix=Path(q['prefix']);input_hashes={str(p):sha(p) for p in prefix.parent.glob(prefix.name+'.input.*')}
 with Path(str(prefix)+'.truck.log').open('w') as log:subprocess.run([str(replay),str(prefix)],stdout=log,stderr=subprocess.STDOUT,check=True,timeout=120)
 result=json.loads(Path(str(prefix)+'.truck.json').read_text())['result'];assert result['status']=='operation-succeeded',(q,result)
 assert result['selected_edges']==1 and all(result[k] for k in ['input_unchanged','geometry_consistent','mesh_closed','step_prepared'])
 with Path(str(prefix)+'.comparison.log').open('w') as log:subprocess.run([str(plugin/'compare_shapes'),str(prefix)+'.result.brep',str(prefix)+'.truck.step',str(prefix)+'.comparison.json'],env=env,stdout=log,stderr=subprocess.STDOUT,check=True,timeout=120)
 compare=json.loads(Path(str(prefix)+'.comparison.json').read_text());assert compare['native_valid'] and compare['truck_step_valid'] and compare['native_solids']==compare['truck_solids']==1
 radius=q['radius'];h=q['offset'];height=q['height'];x=math.sqrt(radius**2-h*h);sign=-1 if q['convex'] else 1
 psi=2*math.asin(q['chord']/(2*radius));phi=math.asin(h/radius)+psi;alpha=math.radians(q['angle'])
 cx,cy=radius*math.cos(phi),radius*math.sin(phi)
 ux,uy=math.sin(phi)*math.cos(alpha)+sign*math.cos(phi)*math.sin(alpha),-math.cos(phi)*math.cos(alpha)+sign*math.sin(phi)*math.sin(alpha)
 px=cx+(h-cy)*ux/uy;assert (px-x)*sign>0
 contacts=[[px,h,0.],[cx,cy,0.]]
 area=radius**2*math.acos(h/radius)-h*x
 removed=abs(x*h-h*px+px*cy-h*cx-radius**2*psi)/2
 expected=((area if q['convex'] else 16*(8-h)-area)-removed)*height
 assert abs(compare['truck_volume']-expected)<1e-7,(q,compare['truck_volume'],expected)
 measured=json.loads(subprocess.check_output([str(root/'target/longitudinal-plane-angle-oracle'),str(prefix)+'.result.brep',str(prefix)+'.truck.step'],env=env,stderr=subprocess.DEVNULL))
 for kind,m in zip(['native','truck'],measured):
  assert len(m['bevel_planes'])==1,m
  plane=m['bevel_planes'][0];n=plane['normal'];o=plane['origin']
  contact=next(p for p in m['vertices'] if p[0]>0 and p[1]>h+1e-8 and abs(math.hypot(p[0],p[1])-radius)<1e-7)
  angle=math.degrees(math.acos(min(1,abs((n[0]*contact[0]+n[1]*contact[1])/radius))))
  chord=math.dist(contact,[x,h,0.]);assert abs(chord-q['chord'])<1e-7
  error=max(min(math.dist(p,w) for w in m['vertices']) for p in contacts)
  q[kind+'_measurement']=dict(**m,contact_angle_degrees=angle,chord=chord,requested_contact_max_error=error)
  if kind=='native':
   # Reproduce the pinned native branch separately from the contact-angle oracle.
   cos_a=-sign*x/radius;sin_a=abs(h)/radius;half=q['chord']/(2*radius)
   sin_half=math.sqrt(1-half*half)*(-1 if cos_a>0 else 1)
   tg=math.tan(alpha+psi/2)
   native_setback=q['chord']*tg/((sin_a+cos_a*tg)*half-(cos_a-sin_a*tg)*sin_half)
   plane_contact=next(p for p in m['vertices'] if 0<p[0]<8-1e-8 and abs(p[1]-h)<1e-8)
   native_model_error=abs((plane_contact[0]-x)*sign-native_setback)
   assert native_model_error<1e-7,(q,native_model_error)
   q['native_source_model']=dict(planar_setback=native_setback,contact_error=native_model_error)

  if kind=='truck':
   assert abs(angle-q['angle'])<1e-8,(q,angle)
   assert error<1e-7,(q,error)
   assert max(abs(sum((p[j]-o[j])*n[j] for j in range(3))) for p in contacts)<1e-7
 native_correct=abs(q['native_measurement']['contact_angle_degrees']-q['angle'])<1e-8
 if native_correct:assert max(compare['native_to_truck_max_distance'],compare['truck_to_native_max_distance'])<.001
 for p,hsh in input_hashes.items():assert sha(p)==hsh,p
 q.update(truck=result,comparison=compare,input_sha256=input_hashes,native_requested_contact_angle_correct=native_correct,expected_contacts=contacts,expected_volume=expected)
 print(prefix.name,'native-angle',q['native_measurement']['contact_angle_degrees'],'truck-angle',q['truck_measurement']['contact_angle_degrees'],'sample',max(compare['native_to_truck_max_distance'],compare['truck_to_native_max_distance']),'material-error',compare['truck_volume']-expected,flush=True)
for p,hsh in v['source_sha256'].items():assert sha(p)==hsh,p
(raw/'validated-summary.json').write_text(json.dumps(records,indent=2)+'\n')
print('All ten Truck controls preserve cylinder contact angle, chord, analytic material and valid single-solid STEP;',sum(q['native_requested_contact_angle_correct'] for q in records),'native controls agree')
