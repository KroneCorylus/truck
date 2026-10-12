# Complete circular arcs and toroidal input round trips

Converting a full circular meridian to a single rational quadratic produced
negative weight and a zero denominator at its midpoint. Imported torus
boundaries consequently failed geometric inclusion. Circle conversion now
uses at most four positive-weight quadratic spans, with shared endpoints
and legal interior knot multiplicities. The conversion contract preserves
the geometric image; additional complete windings do not require extra spans.
The 3D converter lifts every control point and avoids duplicate quarter knots.

After this repair, the E5 horn-torus input exposed a separate STEP orientation
bug. The elementary classifier sampled the singular parameter midpoint;
its NaN normal became a false outward flag and reversed the exported face.
Classification now retries at regular quarter samples and declines elementary
recognition if no finite nonzero orientation is available.

## Validation

Two circle regressions fail on parent 6db16d09, whose kernel source is also
the source at 446ca370. They check complete/nearly complete and large arcs,
small/zero spans, positive weights, exact circular image, accumulated angle,
tangent orthogonality, affine transforms and reversal. Final coverage also
includes decreasing trim ranges. All thirteen circle tests pass unchanged
apart from the added regressions.

All three imported-solid regressions fail the parent's geometry-consistency
check. They now pass for E5 at scale 0.01 and E9 at scales 0.01 and 1, including
rigid motion, consistent geometry, closed topology/meshes, independent volume
and STEP round trips. The horizontal-sector volume derivation and fixture
provenance are in `truck-shapeops/tests/fixtures/toroidal-sectors/README.md`.
A separate parent-failing elementary-surface regression checks horn-torus
orientation under scaling, rigid motion and reversal. The intermediate
circle-only failure is retained to distinguish the two fixes.

Five-crate release validation: **885 passed**, one unchanged PCurve derivative
precision failure, five existing ignored tests, 138 targets. The failing
values and seed match the parent proof retained in
`../2026-10-11-spherical-meridian-step/`. All-target release Clippy and the
action benchmark smoke pass. Clippy requested replacing an equality scan
with `contains`; the final circle target was rerun after that equivalent
change. No existing test was removed, weakened or ignored.

## Construction controls

One warmup and five samples per process, with one/four Rayon workers pinned
to CPU 0 / CPUs 0-3. Processes run sequentially without competing builds or
tests. Fixture import and output validation are outside the operation timer.
All twelve runs succeed and their closed-mesh volumes are identical between
versions. These controls exercise related fillets; they do not measure newly
supported torus fillet construction. Source/binary hashes, machine details,
commands and raw measurements are retained.

| Fixture | Parent median, 1 / 4 workers | Updated median, 1 / 4 workers |
| --- | --- | --- |
| Cylinder | 2.667 / 2.517 ms | 2.530 / 2.463 ms |
| Spline | 25.410 / 25.879 ms | 25.497 / 26.044 ms |
| Sphere | 87.188 / 87.831 ms | 86.550 / 87.511 ms |

The measurements show no material regression in these controls; five warmed
samples are not a tail-latency guarantee. Public signatures and diagnostic
codes are unchanged. This batch establishes repaired toroidal inputs and
round trips. Full captured replay follows the implementation commit, and
broader torus/fillet parity remains unverified.
