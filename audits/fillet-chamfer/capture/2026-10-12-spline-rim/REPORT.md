# Captured replay after closed spline-rim continuation

The full run at clean 99c9d487 (kernel change f7f8f4f5) passes all 156
native entries. Its 161 captures contain 73 constructed Truck results,
31 construction failures, 25 import failures, 23 inconsistent inputs,
eight unhandled intents and one incomplete capture. There are no timeouts
or panics.

J9 is the sole construction-status change from the preceding exact-join
capture. All 73 constructed results preserve input and geometric consistency,
have closed Truck meshes, prepare STEP and yield native-valid single solids.
Seventy-one meet the combined limited checks, including sampled boundary
distances in both directions no greater than 0.001. G1 and tolerance-grid
A1 retain their previously recorded shape discrepancies.

J9's radius-10 spline-pipe rim now follows the displaced longitudinal
side-face seams. Its native-to-Truck maximum sampled distance is
0.0000941988607169 at 574 stations; Truck-to-native is 0.0001098785933683
at 1078 stations. Truck STEP volume is 1808730.6615535542, compared with
native 1808730.7853721464 and independently integrated ideal-sweep volume
1808730.9623989318. The captured input spline is an approximation of that
ideal tube. Truck's checked mesh volume is 1808712.393059274.

Parent-failing regressions also cover radius 5, scale 0.01, rigid motion,
reversed selection and independent rim reparameterization. They check
radius/normals, independent volume, consistent closed topology/meshes,
STEP and immutable input. Final full release tests record 873 passes,
one unchanged PCurve numerical property failure and five existing ignored
tests; Clippy, action smoke and catalog tests pass. Construction medians
are approximately 25–27 ms for this successful spline rim, with the cylinder
control remaining approximately 2.4–2.6 ms. See
`truck-benchmarks/results/2026-10-12-spline-rim-continuation/REPORT.md`
for exact protocols, raw measurements, derivation and validation logs.

Every case artifact was verified against its SHA-256 before archiving.
The manifest retains clean source, replay binary, native runtime and
harness provenance. These sampled comparisons and targeted regressions
are limited evidence; broader construction/import gaps and all full
feature-parity verdicts remain open.
