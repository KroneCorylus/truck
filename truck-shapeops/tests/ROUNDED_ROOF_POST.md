# A post fillet reaching a rounded roof

The fixture is OCCT blend/simple/Q4 capture 2 from the clean 012ace79 replay,
archived in audits/fillet-chamfer/capture/2026-10-11-smooth-fillets/artifacts.zip.
A 5-cube has its x=5,z=5 edge rounded at radius 1. A radius-1 post centered at
(2.5,2.5) projects from z=5 to z=12.5. The selected edge is its circular root.
Input volume is 125 - 5*(1-pi/4) + 7.5*pi = 147.48893571891057.

For a root fillet of radius r that fits the flat roof, added volume is
pi * (2*r^2 + 5*r^3/3 - pi*(1+r)*r^2/2). The test checks r=0.2 against this
formula with the common solid and STEP harnesses.

At r=1, the nominal roof contact extends to x=4.5 but the flat roof ends at
x=4. The original kernel returns intersecting planar loops; 67cf761f rejects
that invalid result. Continued construction now splits the contact at
alpha=acos(0.75) and 2*pi-alpha, rolls onto the existing cylindrical support,
and reconnects the roof's former outer and inner loops into one boundary.
The stronger regression requires successful construction in place of the
intermediate rejection contract. It retains input checks and adds independent
radius, normals, volume, three scales, transforms, reversed selection, closed
meshes and STEP round trips.

## Independent geometry and volume

For azimuth theta, set q=max(cos(theta)-0.75,0), h=sqrt(1-q^2). The ball center
is C=(2.5+2*cos(theta),2.5+2*sin(theta),4+2*h). Support normals are
n0=(cos(theta),sin(theta),0) on the post and n1=(q,0,h) on the roof. Contact
points are C-n0 and C-n1. The blend is the unit circular arc between those
contacts, in the plane normal to the center-curve tangent.

For beta=acos(n0 dot n1), use
p(u,theta)=C-[sin((1-u)*beta)*n0+sin(u*beta)*n1]/sin(beta).
Integrate p.z*(p_u.x*p_theta.y-p_u.y*p_theta.x) over u in [0,1] and one turn
of theta. Subtract the original roof beneath the contact footprint using
Green's theorem, the boundary integral of G(x)dy, where G'=roof height:

- G(x)=5*x for x<=4.
- G(x)=20+4*d+(d*sqrt(1-d^2)+asin(d))/2, d=x-4, otherwise.

Restore the existing post disc contribution 5*pi. Add this difference to the
input volume. The integration is split at both support changes with the
correct one-sided derivatives; doubled Simpson grids must agree within 1e-7.
The independently integrated result is approximately 149.083817359215.

## Native comparison and remaining scope

Native OCCT retains a major-radius-2, minor-radius-1 torus over the flat-roof
arc and constructs two BSpline patches over the rounded roof. At azimuth zero
the post contact is (3.5,2.5,5.9364916731), and the roof contact is
(4.25,2.5,4.96824583655). Its result has 12 faces and volume
149.08377356777387. Different patch counts and representations are acceptable.

The targeted Truck export is native-valid with one solid, volume
149.08379605764409, and bidirectional sampled boundary distances below 0.00033.
This is a limited comparison at tolerance 0.001. Open-wire propagation,
multiple different neighboring supports within one original edge, propagation
through holes, distant collisions and downstream edits of this result remain
outside this regression's verified scope.
