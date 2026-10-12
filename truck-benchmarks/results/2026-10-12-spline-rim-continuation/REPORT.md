# Closed spline-rim contact continuation

The imported OCCT J9 pipe already formed a valid closed solid, but its fillet
failed during face trimming. The side contact at the original rim endpoint
was approximately 0.435 model units away from the longitudinal side-face
seam. The guide must continue onto the neighboring rim segment before that
contact reaches the seam.

Closed-chain construction now solves those one-sided seam crossings and
continues the guide through the adjacent oriented curve with matching
parameter speed. The radius law follows the same neighboring wire interval.
For a displaced seam, contact fitting continues from nearby solved sections,
keeping one parameter branch even when a closed NURBS support has no declared
period. Nominal seam stations that differ only by geometric roundoff stay
unchanged. Open-chain contact initialization is unchanged.

Two end-to-end regressions fail on parent 4a156c4f: the radius-10 original
pipe and a radius-5 pipe scaled by 0.01, rotated, translated and selected in
reverse, with one rim segment independently reparameterized. They check
section radius, contact and radial normal alignment, consistent geometry,
closed topology and meshes, independent physical volume, STEP round trips
and immutable inputs. The independent ideal-tube integration and convergence
are retained beside the fixture. A companion contact-solver regression passes
on the parent and preserves valid support extensions outside the nominal
spline interval; parameter bounds are not face trims.

An initial full-suite run caught an existing transformed reversed-cone STEP
regression when continuation was applied to stationary seams. Restricting
continuation to displaced seams restores that unchanged test. The initial
run is retained as release-tests.txt; release-tests-final.txt records the
final source. No existing test was deleted, ignored or loosened.

Final release geometry/modeling/meshalgo/shapeops/STEP checks: **873 passed**,
one pre-existing PCurve derivative property failure, five existing ignored
tests, 136 targets. The PCurve parent evidence remains in
`../2026-10-11-spherical-meridian-step/`. All-target release Clippy for the five
crates plus benchmarks, action smoke, 19 catalog Python tests and two capture
Tcl tests pass.

## Construction timings

After all builds and tests finished, each process performed one warmup and
five timed calls. Runs were sequential, pinned to CPU 0 or CPUs 0-3, with
one or four Rayon workers. Fixture import, output validation, tessellation
and result destruction were outside construction timing. The benchmark
checks unchanged inputs and closed, geometry-consistent successful output.
Raw samples, commands, source/binary hashes and machine/compiler metadata
are retained here.

| Fixture | Parent median, 1 / 4 workers | Updated median, 1 / 4 workers |
| --- | --- | --- |
| Cylinder rim | 2.645 / 2.431 ms, succeeds | 2.463 / 2.416 ms, succeeds |
| Spline-pipe rim | 44.964 / 45.450 ms, fails trimming | 25.372 / 26.539 ms, succeeds |

The cylinder's checked mesh volume is identical. No speedup ratio is assigned
to the failed spline construction; the updated measurement establishes the
cost of the new successful operation. These small warmed samples are not a
tail-latency guarantee.

Public signatures and diagnostic codes are unchanged. This establishes the
tested constant-radius spline-pipe configurations, not general variable-law,
freeform, multi-corner or full OCCT feature parity. A clean full native capture
will separately establish the resulting case-level audit changes.
