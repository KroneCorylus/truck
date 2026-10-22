# Sphere and two-plane fillet corners

Unmodified STEP inputs and native contour selections from OCCT `blend/simple/D5`,
`D9` and `E4`, capture 001 in the 2026-10-22 open-chamfer-terminals audit.
D5/D9 are radius 150 sphere sectors truncated at latitudes plus/minus 60 degrees,
spanning 90/270 degrees. Their selected edges are one meridian, the upper rim and
its adjoining radial edge. E4 spans 90 degrees from the south pole to latitude 60;
its axis and both meridians are selected. Radius 10 is constant on all three edges.

The imported regression retains the captured physical selections, normalizes
solid and radii by 0.01, and checks both selection orders, closed material, STEP
and input immutability. Original-scale replay, STEP validity, stored tolerances
and native distances are checked separately in the corpus capture. An exploratory
repeated original-scale STEP stress test completed D5/D9, then was stopped while
E4 tessellation consumed more than 20 minutes and 8 GB; it is not passing evidence.

The procedural controls cover 90/270 degree truncated and polar sectors, three
radii, two placements and both selection orders. Their independent material
oracle is a radius-r dilation of an inward radius-(R-r) ball clipped by the two
selected offset planes, then clipped by the unselected terminal planes. Distance
to the inward convex core is computed by enumerating its active constraints;
Gauss-Legendre quadrature integrates the remaining material. 512/1024-node results
agree within 1.3e-7 in unit-scale volume. The extra 180 degree sector is a separate
analytic circular-section integral. Oracle source and convergence logs are
archived with the proof. Nine shared boundaries are checked for tangency, all
fillet radii are exact, and oversized cuts preserve the input on rejection.

D9/E4 retain native sampled discrepancies; native corner patches need not be
exact rolling spheres. E4 additionally exposes inconsistent OCCT surface-volume
integration on the exported STEP; native tessellation agrees with Truck's raw
mesh and the independent material integral within the chord budget. These issues
remain explicit without changing comparison thresholds or claiming parity.
