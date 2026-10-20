import json,math
from pathlib import Path
p=Path('target/native-parallel-cylinder-angles-complete/analysis.json');records=json.loads(p.read_text())
for q in records:
 r=[q['radius1'],q['radius2']];d=q['centers_distance'];x=(r[0]**2-r[1]**2+d*d)/(2*d);y=math.sqrt(r[0]**2-x*x);side=q['reference'];other=1-side;centers=[0.,d];sigma=1 if q['joined'] else -1
 theta=math.atan2(y,x-centers[side]);sweep=2*math.asin(q['chord']/(2*r[side]));rotation=sigma*(1 if side==0 else -1);phi=theta+rotation*sweep
 radial=[math.cos(phi),math.sin(phi)];contact=[centers[side]+r[side]*radial[0],r[side]*radial[1]];tangent=[-rotation*radial[1],rotation*radial[0]];alpha=math.radians(q['angle']);direction=[-tangent[j]*math.cos(alpha)+sigma*radial[j]*math.sin(alpha) for j in range(2)]
 offset=[contact[0]-centers[other],contact[1]];b=sum(a*v for a,v in zip(offset,direction));c=sum(v*v for v in offset)-r[other]**2;disc=b*b-c
 candidates=[]
 if disc>=0:
  for t in [-b-math.sqrt(disc),-b+math.sqrt(disc)]:
   pt=[contact[j]+t*direction[j] for j in range(2)];theta_other=math.atan2(y,x-centers[other]);radial_other=[(pt[0]-centers[other])/r[other],pt[1]/r[other]];old=[math.cos(theta_other),math.sin(theta_other)];rotation_other=sigma*(1 if other==0 else -1);tangent_other=[-rotation_other*old[1],rotation_other*old[0]]
   if t>0 and sum(a*v for a,v in zip(radial_other,old))>0 and sum(a*v for a,v in zip(radial_other,tangent_other))>0:candidates.append(pt)
 if q['native'].get('native_valid'):
  assert candidates,(q,candidates,disc)
  candidates=candidates[:1]
  error=max(min(math.dist(pt+[0.],v) for v in q['measurement']['vertices']) for pt in [contact,candidates[0]])
  assert error<1e-7,(q,error)
  q['circle_model']=dict(contact=contact,other_contact=candidates[0],max_contact_error=error,discriminant=disc)
  print(q['source'],side,q['angle'],'model error',error)
 else:
  assert not candidates,(q,candidates)
  q['circle_model']=dict(discriminant=disc,candidates=candidates)
  print(q['source'],side,q['angle'],'no local circle intersection',disc)
p.with_name('model.json').write_text(json.dumps(records,indent=2)+'\n')
