# Fillets whose curved trims exceed the contact range

A quarter cone occupies x,y≥0 with base radius R and height H. A constant-radius
round r removes material around the axial edge. Its cylindrical surface has center
(r,r) in every horizontal cross-section. The planar support contacts terminate at
z=H(1-r/R), but the middle of the conical end trim rises to
`H(1-(sqrt(2)-1)*r/R)`. Fitting only between contact endpoints drops real geometry.

`curved_fillet_step_domain.rs` constructs this solid directly with shared topology.
Four operations cover R=5, H=12, r=0.25/1, original coordinates/selection and a
rigidly transformed reversed selection. They check input and output consistency,
closed topology/meshes, requested cylindrical radius and normals (including planar
contacts), one generated face, immutable input and STEP round trips. At 174 stations
on each generated-face boundary, the closest point constrained to the prepared
surface's finite parameter rectangle must agree within 0.0001. These stations differ
from the export fitter's sampling grid. Source modeling/meshing tolerance is 0.001;
STEP fitting is 0.00005.

For an independent volume reference, use polar coordinates in the removed planar
corner. The circular boundary has radial distance

`q(theta) = r / (cos(theta)+sin(theta)+sqrt((cos(theta)+sin(theta))²-1))`.

The removed area is `A=r²(1-pi/4)` and its first radial moment is
`M=(1/3) integral[0,pi/2] q(theta)³ dtheta`. Integrating the exact conical height
`H(1-rho/R)` gives remaining volume

`V = pi R² H / 12 - H A + (H/R) M`.

The test evaluates this analytic one-dimensional integral with Simpson's rule,
using `theta=(pi/4)(1-cos(phi))` to remove the endpoint square-root singularity.
At the native R=50, H=120, r=10 dimensions, 512 and 2048 intervals agree within
2e-7 volume units. The 2048-interval reference is approximately 76158.86301257.

The captured upstream `blend/simple/C7` task uses those native dimensions. Its
original Truck STEP loses about 540.48 volume units and has a 4.2283-unit sampled
boundary discrepancy from OCCT. Extending the fitted domain to the actual trims
reduces the bidirectional sampled discrepancy below 2.76e-6, with valid single
solids and a STEP volume difference about 0.000225. These are finite sample and
integration comparisons, not certified global equivalence.

Export now normalizes the procedural surface before finding boundary parameters,
measures the prepared spatial boundary curves in that coordinate system, and fits
the measured longitudinal extent with one sample-step margin beyond nominal ends
that are exceeded. It preserves the original surface, topology and STEP fitting
tolerance. The boundary and fitting checks remain sampled; highly localized extrema
between stations are not globally certified.
