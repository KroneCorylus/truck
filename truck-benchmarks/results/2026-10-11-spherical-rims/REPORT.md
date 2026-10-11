# Spherical rim validation

Kernel baseline: `cc5be492`; audit baseline: `c70452f4`. This batch adds verification,
not a production fix. No existing tests changed, no public APIs or dependencies were
added, and no speedup is claimed. The earlier complete four-crate validation remains
recorded in the conical-rim report; this batch runs only the new integration target.

`spherical_rim_blends` passes in release mode: two tests, 28 operations. The exterior
matrix has two top heights, three exact representations, native/rigid coordinates and
either wire orientation. Four cavity cases cover Sphere and NURBS supports. Checks
include independent radius, normals, G1 joins, analytic volume, topology, closed mesh,
history, unchanged inputs and STEP. See `truck-shapeops/tests/SPHERICAL_RIMS.md` for
the formulas and tolerances. The new target also passes Clippy with `-D warnings`.

The initial analytic/revolution baseline and the initial NURBS baseline both passed.
`strong-regression.txt` and `regression.txt` record a test-fixture error during setup:
calling the vector's `inverse()` reversed face order instead of complementing the
cavity shell. The input-volume check caught this before the blend. Constructing the
fixture with `Solid::not()` fixes its orientation without changing kernel behavior.
The corrected cavity passes in release and debug mode; `release-regression.txt`
records both final tests together. No assertions were relaxed.

The paired OCCT 7.9.3 probe returns valid single solids for all four native analytic
tasks. Radius residuals are approximately four millionths of a model unit, and volume
differences from the exact formula are at most `2.22e-6`. These values are recorded
directly; exact surface identity is not claimed. The probe does not establish OCCT
G1, transformed/NURBS input, mesh or STEP behavior. General curved support pairs and
topology transitions remain open. App code is unchanged and app tests were not run.
