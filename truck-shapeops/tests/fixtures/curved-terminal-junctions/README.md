# Curved terminal junctions

Unmodified STEP inputs and native contour selections captured from OCCT
`blend/simple/B1`, `B5`, `B9` and `C4` in the 2026-10-22 spline-pcurve audit.
B1/B5 are cylinder sectors (radius 50, height 100); B9/C4 are cone sectors
(bottom radius 90, top radius 40, height 150). B1/B9 span 90 degrees;
B5/C4 span 270 degrees. The selected edges are the central axis and the two
upper radial edges, with fillet radius 10. The input is rotated 90 degrees.

Expected material comes from horizontal section integration, independently
of either blend implementation. Write k = 1 - pi/4, R(z) for the outer radius,
and r for the fillet radius. Below height H-r, section area is pi*R^2/4-k*r^2
for convex corners and 3*pi*R^2/4+k*r^2 for concave corners.

In the top strip, z = H-r+r*sin(theta), rho = r*cos(theta), a = r-rho.
Subtract R^2*asin(a/R)+a*sqrt(R^2-a^2) from the corresponding disk-sector area.
The convex corner correction is a^2-k*rho^2; the concave correction is
-a^2+k*(r+a)^2. Integrate using dz = r*cos(theta)*dtheta. The procedural
regression uses Simpson quadrature with 4096 intervals; 1024/4096/16384
interval convergence and native material comparisons are archived with proof.

Tests cover both orientations of the physical selection. Exact constructed
solids use the shared closed-solid/material harness. Normal-budget STEP uses
the established robust round-trip harness; an additional tight-budget export
also passes the strict closed-mesh harness. No audit export tolerance changes.
