# Conical rim blends: validation

Parent: `886bf932` (2026-10-11). This batch fixes revolved-surface inclusion,
periodic fillet/chamfer contacts and procedural STEP boundary fitting. It adds no
dependencies, unsafe code or public API. No speedup is claimed; the benchmark
suite was run in smoke mode with one Rayon worker.

The conical regression covers 32 exterior operations: both cone draft directions,
analytic/exact NURBS supports, native/rigidly transformed coordinates, and both wire
orientations, with a radius-0.5 fillet and unequal-distance chamfer per configuration.
Four more operations cover reversed cavity shells. Independent dimension equations,
normal orientation and analytic volumes accompany topology, closed meshes, complete
face history, immutable inputs and STEP round trips. The formulas and tolerances are
in `truck-shapeops/tests/CONICAL_BLENDS.md`.

Validation:

- Full `cargo test -p truck-geometry -p truck-modeling -p truck-stepio -p truck-shapeops`:
  **723 passed, 0 failed, 4 existing ignored**, including doctests. By crate: geometry
  251 passed/1 ignored; modeling 85/2; shapeops 307/1; STEP 80/0.
- The conical integration target also passes in release mode: 2 tests, 36 operations.
- Modeling regressions independently check line, spline, rational and circular
  generatrices under rigid motion, scaling, shear and reversed orientation, plus
  wrong-frame and singular-transform rejection.
- A geometry regression fits seam-crossing and full-turn plane/cone contacts against
  exact physical contact circles. A STEP regression checks a degree-six procedural
  bulge at 4,097 stations after the initial adaptive fitter misses it.
- Clippy for all targets of geometry, modeling, STEP, shapeops and benchmarks passes
  with `-D warnings`. The action benchmark smoke run passes.
- Targeted nightly formatting and `git diff --check` pass.

Failure evidence is retained separately from final validation. `parent-cone-regression`
and `parent-modeling-regression` reproduce the old coordinate-system error.
`parent-periodic-regression` fails the direct periodic fitter test on the old code.
`pre-fix-chamfer-regression` restores only the old chamfer sampler and fails contact
validation after the other fixes. `cone-regression` and `pre-fix-step-regression`
record the later STEP preparation failure before the export fix.
`parent-curve-fit-regression` restores only the old exporter with the new bulge test
and fails. Their matching passing logs and `full-tests.txt` cover the final sources;
`sources.txt` records hashes. No existing test was removed, ignored or weakened.

The paired OCCT 7.9.3 probe in `audits/fillet-chamfer/probes/conical_blends.cpp`
produces valid single solids with matching dimensions and independent volumes on all
eight native-coordinate analytic exterior/cavity tasks. It does not establish OCCT's
NURBS, transform, reversed-selection, mesh or STEP behavior. Oblique intersections,
apex collapse and general curved support pairs remain open. App code is unchanged;
app tests were not rerun for this batch.
