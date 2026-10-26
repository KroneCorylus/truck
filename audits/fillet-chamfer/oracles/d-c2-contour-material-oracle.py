"""Independent section integration; physical distances, no kernel outputs."""
import math,json
# Profile has two straight lengths8 and quartercircle R2, height10.
# At top-depth w, second chamfer removes inset t=d-w; first cut x<a-w.
def oracle(a,d):
 m=min(a,d)
 overlap=a*d*m-(a+d)*m*m/2+m**3/3
 base=10*(96+math.pi);first=5*a*a
 wedge=(16+math.pi)*d*d/2-math.pi*d**3/12
 return dict(a=a,d=d,base_volume=base,input_volume=base-first,second_wedge=wedge,overlap=overlap,result_volume=base-first-wedge+overlap,terminal_wall_x=max(0,a-d),terminal_wall_z=10-d)
print(json.dumps([oracle(.2,d)for d in [.1,.2,.5]],indent=2))
