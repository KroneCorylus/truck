import math
r=0.5
x=lambda t:30*t*(1-t)**2
y=lambda t:30*t*t*(1-t)
dx=lambda t:30*(1-4*t+3*t*t)
dy=lambda t:30*(2*t-3*t*t)
f=lambda t:x(t)-y(t)-r*(dx(t)+dy(t))/math.hypot(dx(t),dy(t))
a,b=0.,0.1
assert f(a)<0<f(b)
for _ in range(100):
 m=(a+b)/2
 if f(m)<0:a=m
 else:b=m
t=(a+b)/2
c=x(t)-r*dy(t)/math.hypot(dx(t),dy(t))
assert abs(c-(y(t)+r*dx(t)/math.hypot(dx(t),dy(t))))<1e-14
start=math.atan2(x(t)-c,y(t)-c)
end=math.atan2(y(t)-c,x(t)-c)
while end<start:end+=math.tau
arc=(r*c*(math.sin(end)-math.sin(start)-math.cos(end)+math.cos(start))+r*r*(end-start))/2
removed_legs=900*(t**3/3-t**4/2+t**5/5)
area=15-removed_legs+arc
print('contact_parameter',repr(t))
print('center_xy',repr(c))
print('contact_xy',repr(x(t)),repr(y(t)))
print('fillet_angle',repr(end-start))
print('area',repr(area))
print('height_10_volume',repr(10*area))
