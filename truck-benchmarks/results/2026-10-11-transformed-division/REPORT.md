# Transformed curve division

Captured OCCT blend inputs F8, F9, G1, G2 and Z1 panicked before blending because
STEP preprocessing divided the world tolerance by a large transform scale, then
rejected the resulting valid local tolerance below 1e-6. Independent regression
also found that the old orthogonalized scale ignored shear: a circle deviated
0.00855 from chords requested at 0.001.

Curve division now accepts positive finite local tolerances. Affine processors
bound the full linear transform using the absolute row sums of its Gram matrix;
this covers shear and singular projections. Reversed parameter ordering remains
unchanged. Exact straight spans terminate even if squaring a tiny tolerance
underflows. Surface division and projective transforms are outside this change.

The original two tests fail on the parent (panic and excessive chord error).
The expanded four-test matrix covers 2D/3D, reversed curves, uniform/nonuniform
scale, shear, flattened transforms, parabolas, hyperbolas and B-splines, checking
17 independent stations per segment. Full release geometry, geotrait, modeling,
shapeops and STEP suites pass: **747 passed, 0 failed, 5 existing ignored**.
All-target Clippy and the one-worker action benchmark smoke pass. An existing
property-test helper needed the current Clippy equivalent `as_chunks` iteration;
its assertions are unchanged. No API signature, diagnostic, dependency or unsafe
code changed. No comparative performance claim is made. App code is unchanged.

Fresh native captures preserve the five native successes. F8/F9/G1/G2 now return
an unsupported-conic import error instead of panicking. Z1 exposes a separate
odd-divisor index panic in closed-face splitting. None is classified as a working
Truck blend by this fix. Replay evidence is retained in
`target/occt-capture-division-fixed` and is to be attached to the audit separately.
