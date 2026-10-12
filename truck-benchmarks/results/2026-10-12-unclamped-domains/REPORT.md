# Active domains for imported STEP splines

Unclamped STEP B-spline curves and surfaces now use the active knot intervals
when converted into the STEP geometry enums. Knot tails outside those intervals
do not form a partition of unity and could produce zero homogeneous weights
and NaN points. Curves are cut to their active interval; surfaces are cut in
both parameter directions. Rational weights are applied before cutting, so
the geometry and parameterization within the active domain are preserved.
Low-level conversion of raw STEP control data is unchanged.

Both tests in truck-stepio/tests/unclamped_curve_domains.rs fail on 0d8bface
and pass with the fix. They cover polynomial and weighted rational curves in
2D/3D, reversed edge orientation, surfaces with padding in both knot vectors,
independent analytic positions/normals and finite bounded curve sampling.
Existing tests are unchanged. Full release STEP/shapeops: 422 passed, zero
failed, one existing ignored, 76 targets. All-target release Clippy for STEP,
shapeops and benchmarks, and the action benchmark smoke check pass.

Captured blend/simple/J9 previously spent its 120-second replay budget in
recursive subdivision of invalid NURBS edge intervals. Its two closed edges
had full knot extents approximately [-2pi,4pi] but active domain [0,2pi], and
the pipe surface had the same padded periodic direction. The targeted replay
now completes preprocessing and rejects a separate unsupported imported
spline pcurve. This is not a successful fillet or a full parity result.
The exact input hashes, source parent and pre-commit replay scope are recorded
in j9-provenance.json. A fresh clean capture is recorded separately.

This fixes valid imported spline domains; it does not make arbitrary invalid
curves safe for non-fallible generic subdivision or add spline seam conversion.
No comparative performance claim is made between the timeout and rejection.
