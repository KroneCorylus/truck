import mpmath as m
for digits in [40,80]:
 m.mp.dps=digits
 end=m.mpf('.95');a0=m.mpf('.25');length=m.mpf('1.2')
 initial=m.pi/4*(1+m.mpf('.2')/3*(1+end+end**2)-length*a0*a0)
 for equal in [False,True]:
  tangent=m.mpf('.2') if equal else m.tan(2*m.asin(m.mpf('.1')))
  psi=m.atan(tangent);a=m.mpf('.8') if equal else m.mpf('.9')
  alpha=m.atan(m.sin(psi)/(m.cos(psi)-a))
  def removed(r):
   t=-a*m.cos(alpha)+m.sqrt(r*r-a*a*m.sin(alpha)**2)
   y=a+t*m.cos(alpha);z=t*m.sin(alpha);p=m.atan2(z,y)
   return (r*r*p-a*z)/2
  results=[]
  for method in ['tanh-sinh','gauss-legendre']:
   cut=removed(1)+m.quad(lambda x:removed(1-(x-1)/4),[1,length],method=method)
   results.append(initial-cut)
  assert abs(results[0]-results[1])<m.mpf(10)**(-digits+5)
  print('digits',digits,'equal',equal,'tangent',m.nstr(tangent,digits),'slope',m.nstr(m.tan(alpha),digits),'input',m.nstr(initial,digits),'remaining',m.nstr(results[0],digits))
