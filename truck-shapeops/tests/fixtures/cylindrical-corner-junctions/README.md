# Cylinder rim junctions

Unmodified native input and full contour selections from OCCT blend/simple/B3
and B7, captured at the 2026-10-22 curved-terminal checkpoint. Each input is a
radius-50, height-100 cylinder sector. B3 spans 90 degrees and B7 spans 270.
A radius-10 fillet selects the upper circular rim, one outer axial ruling and
the adjoining upper radial edge. The shared corner joins a cylinder and two
planes. Its exact blends are two cylinders, a torus and a sphere.

For an outer radius R, height H and fillet radius r, put A=R-r,
theta=asin(r/A) and x=sqrt(A*A-r*r). The rolling-sphere center is
(x,r,H-r). At z=H-r+r*sin(phi), set rho=r*cos(phi), a=r-rho and outer=A+rho.
Green's theorem gives the horizontal material area as
(-a*x+x*rho*(sin(theta)+1)-r*rho*cos(theta)+rho*rho*(theta+pi/2)
 +outer*outer*(alpha-theta))/2, with alpha=pi/2 or 3*pi/2.
Below H-r use rho=r. Integrate the upper area against r*cos(phi) dphi.
The independent results are 190760.57996317057 and 580239.2966730265.

Procedural tests cover both sector angles, three radii (including a cut past
the original rim midpoint), rigid/uniform-scale placements, both selection
orders, exact radii, tangent normals, material, closed meshes and STEP.
Oversized requests remain rejected without input mutation. Native replay
checks the full contour in both orientations at ordinary and tight STEP
fitting budgets.
