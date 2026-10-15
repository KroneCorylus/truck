# Consistent small-scale tangent chamfer fits

A rounded prism with a quarter-circle top contour succeeds at large scale but
previously rejects at scale 0.025: its exact support pcurve separates from the
fitted ruled chamfer surface beyond kernel consistency tolerance. This reproduces
the native A4 full-contour failure without using native output as the oracle.

Shared contact fitting now refines only when geometric distance from a support
pcurve to its spatial spline exceeds min(requested tolerance, 1e-7). Nearest
parameters are bounded to the spline domain; equal parameter values are not
required for equivalent curves. Refinement doubles intervals up to 257 samples,
while preserving a larger initial edge-sampling count. The existing consistency,
contact, topology and export validation remains unchanged. The ruled surface normal is evaluated in its own normalized parameter chart,
preventing shifted source parameters from reversing a bevel face.
Interior spline knots now average neighboring sample parameters, preventing
large-sample interpolation instability. The parent loses straight-line accuracy
at 129 samples and diverges by hundreds of units at 257; the new unit regression
checks points and first/second derivatives through 257 samples without loosening
assertions. The intermediate full run had 499 passes and two NURBS-cylinder
failures, retained in affected-before-knot-fix.txt; stable knots resolve both.
No API, dependency or consumer changes. No performance claim.

The first implementation used equal-parameter distance and rejected an existing
spline-cone regression. Diagnostics show distance 5.96e-5 at equal parameters but
geometric distance 6.44e-8. That implementation was corrected to measure geometric
agreement. `affected-initial.txt` and `cone-fit-diagnosis.txt` preserve the failure;
the existing cone tests and all other assertions remain unchanged.
`parent-chart-tests.txt` separately demonstrates that fitting alone leaves a
shifted-parameter closed-mesh failure. `affected-before-chart-fix.txt` is an
explicitly interrupted intermediate validation pass, not a completed pass.

Two new solid regressions and a dense affine interpolation unit test fail on
the parent; a direct simple-chamfer control
passes there and remains passing. The solid regression checks two scales,
rotation/translation, reversed selections, shifted NURBS edge parameters, input immutability, topology, closed
mesh, STEP, analytic bevel positions/normals and independent material. For the
10x20 profile with one R4 corner extruded 30, base volume=(184+4pi)*30.
Integrating the removed strip for equal setback d gives
(22+2pi)*d²/2-pi*d³/12. The test uses d=1 before scaling; it does not copy native
volume. Operation tolerance 0.001 is unchanged; verification uses 1e-4*scale.

Full affected release suites: 502 passed,
0 failed, 1 pre-existing
ignored. All-target release Clippy, action benchmark smoke and 15 replay tests pass.
The final source and replay hashes are in metadata.json.

Native A4 proof retains its original captured seed and raw BREP/STEP separately.
An independent native builder inspector identifies the complete three-edge
line/arc/line contour. Explicitly replaying those physical edges now succeeds,
preserves input, has consistent geometry and a closed mesh, and exports a
native-valid single-solid STEP. Largest sampled distance is
6.1772021936943812e-06;
STEP volume difference is 6.9110611566891578e-08.
These finite samples do not establish a global bound or full parity.

Automatic capture is still seed-only. This proof uses an explicitly expanded
native contour and does not silently relabel old corpus records. Capturing native
propagation and first-assignment semantics remains the next separate audit task.
The preparation probes also found an independent face-trimming panic for a
half-cylinder top bounded by only an arc and a single chord; notes are archived
for a separate parent-failing regression/fix. The face-level control uses distinct
neighbor edges so it tests the intended fitting path.

Native archive: 34 byte-verified members, SHA256 `d446d81bb3b994636df57e5d3bdfbf4c8156167b4479fb16615582d36d1f599e`.
Raw directory target/tangent-chamfer-fit-proof. The initial superseded proof stays
in target/tangent-chamfer-fit-initial-proof. Full clean corpus capture follows the
implementation/proof commit; prior C1 input gaps, curved reference support,
closed-shell workflow, missing fixtures and PCurve precision failure remain open.
