# Mixed curved corner selections

Native blend/simple/F1 and H7 select three edges at one trivalent vertex: one
concave rim and two convex boundaries sharing a planar wall. F1 has a toroidal
support; H7 has a cylinder with a remote conical termination.

The regression preserves the physical selections and checks three radii, two
placements and both orders. It verifies four generated faces, constant rolling
radius, ten tangent joins, independent material volume, STEP and unchanged input.
The procedural F1 controls also retain four oversized-radius rejections.

Material values come from independent divergence-flux quadrature, with no CAD,
native builder or tessellation dependency; derivations and convergence are in the
checkpoint proof. Native comparison remains separate: original-scale H7 has a
recorded discrepancy even though the geometric and material controls pass.
