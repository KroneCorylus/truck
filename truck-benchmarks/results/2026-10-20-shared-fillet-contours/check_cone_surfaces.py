from pathlib import Path
import subprocess,os,json,math
root=Path.cwd();env=dict(os.environ,LD_LIBRARY_PATH=str(root/'target/occt-sdk/usr/lib64'))
def check(q,p):
 surface=json.loads(subprocess.check_output([str(root/'target/cylinder-angle-surface-oracle'),str(p)+'.step'],env=env,stderr=subprocess.DEVNULL))[0];Path(str(p)+'.surfaces.json').write_text(json.dumps(surface,indent=2)+'\n')
 r,k=q['fillet_radius'],1/6;n=math.hypot(1,k);groups=[];area=0.
 for upper in [False,True]:
  if upper:
   c,z=1.5-r*(n-k),8-r;contacts=[(c,8),(c+r/n,z+r*k/n)];a,b=math.atan(k),math.pi/2
  else:
   c,z=2+r*(n-k),5+r;contacts=[(c,5),(c-r/n,z-r*k/n)];a,b=math.atan2(-k,-1),-math.pi/2
  def contact(p,index):return abs(math.hypot(*p[:2])-contacts[index][0])<1e-7 and abs(p[2]-contacts[index][1])<1e-7
  faces=[f for f in surface['faces'] if len(f['vertices'])>=2 and all(contact(p,0) or contact(p,1) for p in f['vertices']) and all(any(contact(p,i) for p in f['vertices']) for i in [0,1])];assert len(faces)==2,(upper,len(faces))
  errors=[abs(math.hypot(math.hypot(*p[:2])-c,p[2]-z)-r) for f in faces for p in f['samples']];assert max(errors)<5e-5,(upper,max(errors));groups.append(dict(upper=upper,faces=len(faces),samples=len(errors),max_radius_error=max(errors)))
  area+=2*math.pi*r*(c*(b-a)+r*(math.sin(b)-math.sin(a)))
 return dict(bevels=groups,analytic_bevel_area=area,bevel_area_times_step_tolerance=area*5e-5)
if __name__=='__main__':
 q=json.load(open('target/native-shared-cone-fillet/summary.json'));p=root/'target/disjoint-fillet-controls/stack';row=check(q,p);comparison=json.loads(Path(str(p)+'.json').read_text());volume_error=abs(comparison['truck_volume']-q['expected_volume']);assert volume_error<row['bevel_area_times_step_tolerance'];row.update(mode=q['mode'],volume_error=volume_error);print(row);(root/'target/disjoint-fillet-controls/stack-radius-material-checks.json').write_text(json.dumps(row,indent=2)+'\n')
