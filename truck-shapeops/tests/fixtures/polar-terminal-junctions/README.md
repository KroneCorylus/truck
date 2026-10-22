# Polar terminal junction

Unmodified captured input and native contour selection for OCCT
`blend/simple/E2`, from the 2026-10-22 curved-terminal audit. The solid is a
radius-150 sphere sector spanning 90 degrees, from its lower pole to latitude
60 degrees, rotated 90 degrees about Z. Three planar edges meet at the upper
axis vertex; their fillet radius is 10. The lower end of the axial edge
terminates at the sphere's parameterization pole.

The independent material integral starts with the quarter spherical segment.
The vertical round removes the planar corner outside the radius-r circle
centered at (r,r). Integrate its height from the lower sphere to the upper
plane over x=r*(1-cos(t)), 0<=y<=r*(1-sin(t)). The remaining upper fillets remove
a second horizontal section strip, without counting the vertical cut twice.
The exact integrands are in `polar_terminal_junctions.rs`; Simpson calculations
at 1024/4096/16384 intervals converge to 3479626.174979871, within 0.00000176
of the captured native material.

The procedural controls check transformed scales, two radii, both selection
orders, unchanged input, exact blend radii, samples on both terminal supports,
closed mesh, material and STEP. Oversized requests remain rejected. Native
replay checks the complete contour, both selection orientations and ordinary
and tight STEP budgets using the shared curved-terminal fixture harness.
