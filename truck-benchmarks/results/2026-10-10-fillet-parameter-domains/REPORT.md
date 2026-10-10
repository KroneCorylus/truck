# Fillet parameter domains during STEP export

STEP preparation now exports a fillet whose longitudinal knot range is scaled to
`1e-9`, instead of rejecting the same physical surface. The fitter normalizes a copy
of the source to `[0, 1]` before evaluating sections and interpolating their homogeneous
control points. It retains contact-knot breaks, the 193-control-point limit and the
existing geometric error samples. Input geometry and topology are unchanged.

`ApproxFilletSurface::try_vknot_normalize` rejects empty, non-finite, unordered,
zero-width and overflowing ranges without mutation. It subtracts the origin before
dividing by the span, avoiding cancellation from separate scale/translation operations.
Other spline evaluators and modeling paths retain their existing parameter constraints.

## Correctness

Baseline production sources are Truck `e34c0701`. The strengthened regression fails
there because `prepare_for_step` returns `None` for a `1e-9` domain. The parent failure
is in `parent-regression.txt`. The existing no-panic and unchanged-input checks remain;
the test now additionally requires successful preparation and a valid STEP round trip.

The regression covers parameter scales `1e-9`, `1e9` and `0.125`, zero and nonzero
origins, and both original and rotated/translated/uniformly-scaled solids. It compares
fitted surface points with the original surface at corresponding normalized parameters,
checks the source solid using the shared topology/closed-mesh/exact-volume harness,
and checks exported topology, robust mesh closure and independent analytic volume.
The reference volume integrates the circular cross-section around the cylinder axis.
Geometry-unit checks cover positions, derivatives and unchanged invalid inputs.

Prepared geometry uses the documented robust tessellator because fitted boundaries and
surfaces may differ by their fitting tolerances. The volume allowance includes that
approximation gap; it does not require exact curve/surface coincidence after export.

Validation:

- Geometry suite, including doctests: 250 passed, 0 failed, 1 existing ignored.
- STEP suite, including doctests: 79 passed, 0 failed.
- Curved representation, wire representation and modeling blend suites: 13 passed.
- All-targets Clippy for geometry, STEP, shapeops and benchmarks passed, as did the
  complete action benchmark smoke run, targeted nightly rustfmt and `git diff --check`.
- Source hashes and toolchain are recorded in `sources.txt`.

These are independent Truck regressions. No OCCT catalog cases were executed, and
this does not establish arbitrary support/curve parameter invariance or general seam parity.

## Export timing

The new `step::prepare_fillet` action measures only STEP preparation of a radius-0.5
rim fillet on a radius-5, height-8 cylinder, at fitting tolerance `0.00005`.
Analytic and exact NURBS supports are measured separately. Fixture construction,
source-volume checks and prepared robust mesh/volume checks run outside the timer.
Production geometry and exporter sources were restored to the baseline for the before
runs and restored to the working implementation afterward. Benchmark code is identical
apart from formatting. There were no simultaneous builds or test runs during timing.

Initial five-sample, unpinned runs measured analytic export at 19.51 → 18.83 ms and
NURBS export at 11.76 → 13.07 ms. The apparent NURBS slowdown prompted three longer
runs per version, each with 20 single-iteration samples, one Rayon worker, pinned to CPU 0.

| Representation | Before run medians (ms) | After run medians (ms) | Median of run medians |
| --- | --- | --- | --- |
| Analytic | 18.80, 19.25, 19.06 | 19.21, 18.72, 18.67 | 19.06 → 18.72 ms (-1.8%) |
| NURBS | 13.38, 13.21, 13.11 | 13.46, 14.04, 13.05 | 13.21 → 13.46 ms (+1.9%) |

Run timings overlap. These local samples do not establish a significant performance
change in either direction. Both initial and follow-up measurements are retained.

```sh
RAYON_NUM_THREADS=1 cargo bench --locked -p truck-benchmarks --bench actions -- step::prepare_fillet
# After cargo bench --no-run, for each before/after binary:
RAYON_NUM_THREADS=1 taskset -c 0 target/release/deps/actions-d92deb45127f2497 \
  --bench step::prepare_fillet --sample-count 20 --sample-size 1
```
