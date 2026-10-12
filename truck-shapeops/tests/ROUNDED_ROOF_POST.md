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
x=4. The old kernel returns intersecting planar loops. Completion now rejects
that invalid result. This rejection is not feature parity.

Native OCCT retains a major-radius-2, minor-radius-1 torus over azimuth
acos(0.75) through 2*pi-acos(0.75). On the remaining arc it constructs two
BSpline patches tangent to both the post and the existing cylindrical roof.
At azimuth zero their post contact is (3.5,2.5,5.9364916731), and roof contact
is (4.25,2.5,4.96824583655). The native result has 12 faces and volume
149.08377356777387. Merely clipping the original torus is insufficient:
the rolling-ball center height must change as support changes.
