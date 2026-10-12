# Periodic torus projection, circular trims and material-preserving splits

J3/J4/J5 previously failed import as nonsimple wires. The torus inverse
normalized a radial vector and applied inverse cosine, which could invent
a 1.49e-8-radian angle even on an exact axis point. At radius 250 this moved
the reconstructed point about 3.7e-6 off the support. Signed `atan2` now
recovers both angles without that amplification and keeps the half-open
periodic domain. The torus normal's u derivative also no longer contains an
incorrect z component.

After projection succeeds, preprocessing creates isoparametric pcurves on
the toroidal support. Those now become exact modeling conics, preserving
both circle directions, orientation, affine transforms and endpoints.
Only machine-roundoff deviations from constant parameters are flattened,
with a conservative bound on the world-space error after transformation.
Real diagonal curves, degenerate circles and nonfinite trims are rejected.

J3 then exposed a separate branch error: its divisor joined the ends of a
270-degree sector along the complementary 90-degree arc. Splitting now
retains the continuous parameter chart when both divisor vertices belong
to one closed UV boundary. Other cases retain periodic alignment. The same
chosen line drives both intersection and edge creation.

## Validation

Two torus regressions fail on the parent implementation and now verify
near-axis/seam point recovery, nearest points, off-surface rejection and
normal derivatives against independent finite differences. Two pcurve
regressions fail on the parent converter and now check positions/tangents,
parameter-speed mapping, full/partial circles, reversal, inverted supports,
affine transforms and bounded seam roundoff. The final rejection checks also
cover extreme transform amplification, a collapsed horn circle and NaN.

Four new solid regressions fail against the complete 88f64f08 production
source, which is also the kernel at eeef4a3d. J3 is checked at scales 0.01
and 1 with a rigid transform in the smaller case; J4/J5 at scale 0.01,
including J5 rotation/translation. They require geometry consistency,
closed topology/meshes, independent volume and STEP round trips. All seven
torus input tests, including unchanged E5/E9 assertions, pass. The shared
volume formula was generalized from a quarter revolution to arbitrary
major angle without changing the existing cases' expected geometry.
The independent formulas and fixture provenance remain beside the STEP files.

Five-crate release validation: **893 passed**, one unchanged PCurve precision
failure, five existing ignored tests, 139 targets. The failing values match
the retained parent proof in `../2026-10-11-spherical-meridian-step/`.
All-target Clippy, including the benchmark package, and action smoke pass.
After replacing a redundant bounded-angle remainder with equivalent sign
normalization, the torus and pcurve targets, Clippy and action smoke were
rerun successfully. Existing assertions were not weakened or removed.

## Import timings and completed validation

One warmup and five measured imports per process. One/four Rayon workers
are pinned to CPU 0 / CPUs 0-3, with sequential processes and no competing
builds or tests. Timing includes STEP parsing, preprocessing and modeling
conversion; file I/O, final-output destruction and geometry/mesh/volume
checks are outside it. Successful outputs are checked at scale 0.01 and
mesh tolerance 0.0001. Parent E5/E9 mesh volumes are identical to updated
volumes. J3/J4/J5 parent failures receive no speedup ratios.

| Case | Parent median, 1 / 4 workers | Updated median, 1 / 4 workers |
| --- | --- | --- |
| E5 | 1.987 / 1.901 ms, passes | 2.486 / 2.339 ms, passes |
| E9 | 2.265 / 2.204 ms, passes | 2.925 / 2.921 ms, passes |
| J3 | 4.783 / 4.683 ms, fails | 16.415 / 17.052 ms, passes |
| J4 | 6.024 / 6.045 ms, fails | 12.362 / 12.599 ms, passes |
| J5 | 5.697 / 5.688 ms, fails | 13.503 / 13.691 ms, passes |

E5/E9 cost about 0.4-0.7 ms more because preprocessing now completes checks
that previously stopped on projection errors. The retained boundary probe
uses the same 0.00001 chord sampling: the parent rejects **144 of 6541**
E5 samples and **387 of 9370** E9 samples; the update rejects zero. The
preprocessor returns early on such an unsuccessful projection. Thus the
old and new paths do not perform equal validation work, even though both
simple inputs happened to produce the same final solids. The extra latency
is recorded explicitly; no speedup or zero-overhead claim is made.

To reproduce the boundary probe, copy `boundary_projection_probe.rs` to
`truck-stepio/examples/torus_boundary_projection.rs`, build that release
example and pass the E5/E9 fixture paths. Its parent run replaces only the
torus primitive with the 88f64f08 source; raw STEP projection does not use
the modeling converter. The import timing parent's entire production source
was verified against 88f64f08 before its executable was retained. Metadata
records source/fixture/binary hashes, compiler, CPU, power mode and commands.
Initial timing records are retained under `initial/`.

Public signatures and diagnostic codes are unchanged. This batch establishes
periodic input/round-trip coverage; clean native blend replay follows the
implementation commit. The H2/H4 ellipsoidal tessellation timeouts are a
separate active investigation and are not marked resolved here.
