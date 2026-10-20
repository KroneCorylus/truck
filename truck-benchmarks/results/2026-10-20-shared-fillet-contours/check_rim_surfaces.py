from pathlib import Path
import subprocess,os,json,math
root=Path.cwd();env=dict(os.environ,LD_LIBRARY_PATH=str(root/'target/occt-sdk/usr/lib64'))
def check(q,p):
 surface=json.loads(subprocess.check_output([str(root/'target/cylinder-angle-surface-oracle'),str(p)+'.step'],env=env,stderr=subprocess.DEVNULL))[0]
 Path(str(p)+'.surfaces.json').write_text(json.dumps(surface,indent=2)+'\n');groups=[]
 for top in [False,True]:
  r,R,H=q['fillet_radius'],q['radius'],q['height'];cap=H if top else 0.;zcenter=H-r if top else r
  def contact(p):return (abs(p[2]-cap)<1e-7 and abs(math.hypot(*p[:2])-(R-r))<1e-7) or (abs(p[2]-zcenter)<1e-7 and abs(math.hypot(*p[:2])-R)<1e-7)
  faces=[f for f in surface['faces'] if len(f['vertices'])>=2 and all(contact(p) for p in f['vertices']) and any(abs(p[2]-cap)<1e-7 for p in f['vertices']) and any(abs(p[2]-zcenter)<1e-7 for p in f['vertices'])]
  assert len(faces)==(2 if q['mode']=='full' else 1),q
  errors=[abs(math.hypot(math.hypot(*p[:2])-(R-r),p[2]-zcenter)-r) for f in faces for p in f['samples']];assert max(errors)<5e-5,(q,max(errors))
  groups.append(dict(top=top,faces=len(faces),samples=len(errors),max_radius_error=max(errors)))
 area=2*q['theta']*q['fillet_radius']*((q['radius']-q['fillet_radius'])*math.pi/2+q['fillet_radius'])
 return dict(bevels=groups,analytic_bevel_area=area,bevel_area_times_step_tolerance=area*5e-5)
if __name__=='__main__':
 rows=[]
 for q in json.load(open('target/native-disjoint-fillet-rims/summary.json')):
  p=root/'target/disjoint-fillet-controls'/q['mode'];row=check(q,p);comparison=json.loads(Path(str(p)+'.json').read_text());volume_error=abs(comparison['truck_volume']-q['expected_volume']);assert volume_error<row['bevel_area_times_step_tolerance'];row.update(mode=q['mode'],volume_error=volume_error);rows.append(row);print(row)
 (root/'target/disjoint-fillet-controls/radius-material-checks.json').write_text(json.dumps(rows,indent=2)+'\n')
