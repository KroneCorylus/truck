from pathlib import Path
import json,math
for name in ['fan','unsplit']:
 root=Path('target/native-cylindrical-terminal-'+name);records=[]
 R,h,L,H=5.,2.,5.,10.;initial=math.pi*R*R*H-L*(h*math.sqrt(R*R-h*h)+R*R*math.asin(h/R))
 for i,p in enumerate(sorted((root/'capture').glob('*.input.json'))):
  q=json.loads(p.with_name(p.name.replace('.input.','.result.')).read_text());dy=.7;dz=[.7,.5,.7/math.sqrt(3)][i//2];a=h-dy
  I=lambda y:(y*math.sqrt(R*R-y*y)+R*R*math.asin(y/R))/2
  J=lambda y:-(R*R-y*y)**1.5/3
  expected=initial+(i%2+1)*dz/dy*(J(h)-J(a)-a*(I(h)-I(a)))
  record=dict(prefix=str(p)[:-len('.input.json')],split=name=='fan',count=i%2+1,mode=['equal','two-distances','distance-angle'][i//2],radius=R,halfwidth=h,level=L,height=H,planar_distance=dy,wall_distance=dz,initial_volume=initial,expected_volume=expected,native=q,native_input_unchanged=(root/f'before{i}.result.brep').read_bytes()==(root/f'after{i}.result.brep').read_bytes())
  if name=='unsplit':assert q['native_valid'] and q['solids']==1 and abs(q['volume']-expected)<1e-7,(record,q['volume']-expected)
  records.append(record);print(name,i,'valid',q['native_valid'],'volume error',q['volume']-expected,'unchanged',record['native_input_unchanged'])
 (root/'analysis.json').write_text(json.dumps(records,indent=2)+'\n')
