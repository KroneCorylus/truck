# Extruded curved corner selection

Native OCCT blend/simple/F5 selects three edges joining an elliptical extrusion,
upper cap, and planar wall. Input STEP and intent preserve the captured physical
selection and original units. Tests normalize by 100 and reverse selection order.

Independent material uses divergence-flux integration with divergence one, not
native CAD or tessellation: normalized radius .1 has volume 3.092505604759356.
The procedural controls also check radii .03/.06, transformed placements, nine
G1 joins, requested radius, closed meshes, STEP and immutable failures.

Short bodies (heights .13 and .15, radius .1) use the same flux oracle with
volumes .391151182497675 and .45325128415886307.

Original-scale native comparison remains a discrepancy (sampled distance about
0.107). Truck's volume matches the independent oracle to 6e-9; native OCCT's
differs by about 2.5 and carries a 0.022 face tolerance.
This establishes the tested elliptical extrusion, not generic extrusion parity.
The solver retains absolute residual/derivative thresholds (1e-9 through 1e-15)
and unwraps periodic branches at 32 stations. Extreme scales, singular or folded
offsets, and high-winding profiles need separate evidence; finite guards do not
establish a solution or guarantee the correct periodic branch.
