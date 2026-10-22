# Unequal convex chamfer transitions

Unmodified native input and full selection from OCCT chamfer/equal_dist_complex/A1,
captured by the 2026-10-22 cylindrical-corner checkpoint. Two convex bevel pairs
with unequal setbacks leave transition triangles. The previous normal-sum rule
oriented those triangles in the same direction as their adjacent boundaries.

The regression checks successful construction, all 17 selected edges, unchanged
input, consistent closed geometry, native material within the standard harness
mesh budget, and STEP. It does not claim a native comparison pass: the full case
still differs in corner geometry, with sampled distances near 0.0212 and 0.0289.
Independent material proof is supplied by the reduced procedural bracket tests.
For perpendicular equal-side setbacks a and b, separate the cut prisms by
z=b*x/a: removed volume is H*a*a/2 + L*b*b/2 - a*b*(a+b)/6.
