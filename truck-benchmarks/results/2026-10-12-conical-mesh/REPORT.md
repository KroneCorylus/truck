# Conical blend mesh and tight spline fitting

I3's conical end trims caused excessive refinement in raw UV space:
one surface direction is an angle, the other a generator distance. Where only
one axis has interior grid divisions, Delaunay refinement now uses a centered,
normalized diagonal metric from the surface derivative lengths. Geometry,
trim inclusion, shared boundary positions and physical error checks retain
the original parameters. Faces with a two-direction grid retain their prior
triangulation policy, including the existing thin-loft accuracy checks.

The scaled/rotated STEP regression also exposed a separate precision loss:
cubic fitting was followed by knot removal using the global tolerance,
which could undo the requested tighter fit. Fitting now retains its spline
joins. The standalone position/derivative regression and conical solid
regression both fail on parent 713f0c9a. No existing test was changed.

The solid regression checks original size and scale 0.01 with rigid transform
and reversed selection. Both have consistent geometry, bounded coarse
meshes, closed fine meshes, correct topology, independent volume, STEP round
trips and immutable inputs. The independently integrated volume is
979697.406914389, versus native 979697.406915526. The fixture README gives
the integral and provenance.

Full release geometry/modeling/meshalgo/shapeops/STEP: **868 passed**, one
unchanged PCurve derivative property failure, five existing ignored tests,
135 targets. The PCurve failure reproduces the previously recorded large
coordinate rounding case; its parent proof and seed are in
`../2026-10-11-spherical-meridian-step/`. All-target release Clippy and the
action benchmark smoke check pass. Public APIs and diagnostics are unchanged.

## Measurements

`truck-benchmarks/examples/blend_output.rs` uses the same retained serialized
I3 blend for both binaries. One warmup precedes five samples. Setup and
post-mesh validation are outside each timer. Processes run sequentially,
pinned to CPU 0 or CPUs 0-3, with respectively one or four Rayon workers.
No builds or test suites ran during measurement. Source/binary/fixture
hashes, raw samples, compiler and CPU metadata are retained beside this report.

| Action | Parent median, 1 / 4 workers | Fixed median, 1 / 4 workers |
| --- | --- | --- |
| Mesh, tolerance 10 | 467.241 / 268.254 ms, **open** | 0.044 / 0.086 ms, closed |
| Mesh, tolerance 0.001 | 30 s process timeout / timeout | 416.489 / 164.231 ms, closed |
| STEP, tolerance 0.00005 | 1.448 / 1.481 ms, succeeds | 1.430 / 1.420 ms, succeeds |
| STEP, scale 0.01, tolerance 0.0000005 | 2.496 / 2.477 ms, **fails** | 1.435 / 1.442 ms, succeeds |

Failures and timeouts do not earn speedup ratios. The timeout includes setup
and warmup and is not an operation-latency lower bound. The coarse mesh falls
from 67959 vertices to 14 while gaining closure; the fine fixed mesh has
80888 vertices and volume 979691.1574499073, within the unchanged physical
chord-error allowance. The STEP fit retains 128 spline controls versus 64
on the previously successful original-scale export; this storage tradeoff
preserves the tighter fit without a measured preparation slowdown here.

To reproduce, extract `benchmark-fixture.zip`, build the example on parent
and fixed sources, then run each binary with arguments
`I3.solid.json mesh 0.001 1` or `I3.solid.json step 0.0000005 0.01`, setting
`RAYON_NUM_THREADS` and CPU affinity as above. `measurements.json` retains
all executed commands and both coarse and fine settings.

This establishes the regression and bounded completion for this fixture.
The subsequent clean full capture records native shape comparisons and
remaining gaps; no complete fillet/chamfer parity is claimed here.
