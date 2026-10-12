# Exact spline seam import

Linear pcurves with one constant parameter on B-spline/NURBS supports now
convert to exact spatial splines. The fixed parameter is evaluated in the
polynomial or homogeneous control net, then the resulting curve is trimmed,
oriented and parameterized on [0,1]. Seam-search roundoff is accepted within
32 floating-point epsilons of the coordinate scale; real diagonal changes or
support overshoots are rejected. Existing planar/cylindrical routes and public
APIs are unchanged.

All three new regressions fail on parent 98f1fe32. Two check polynomial and
rational sections, both parameter directions, added interior knots,
non-unit domains, position/first/second derivatives, rigid transformation,
reversal and roundoff/rejection boundaries. The imported J9 pipe regression
checks consistent geometry, topology, closed mesh, independent swept-disk
volume, STEP and immutable input at two scales with a rigid transform.
The ideal volume is 1815077.75712917; the source's approximated native pipe
has volume 1815077.51635755. Fixture provenance and the quadrature are retained
with the test. No existing test was changed.

Full release STEP/shapeops: 425 passed, zero failed, one existing ignored,
78 targets. All-target release STEP/shapeops/benchmarks Clippy and the action
benchmark smoke check pass. No performance improvement is claimed.

J9 now imports and passes input consistency and physical selection. Its two
selected rim segments reach the blend solver, which still reports
BlendConstructionFailed at trim_faces with unchanged input. The targeted
replay's source scope and input hash are recorded in j9-provenance.json;
this is not successful filleting or full case parity. General diagonal or
nonlinear pcurves on spline supports remain outside this conversion.
