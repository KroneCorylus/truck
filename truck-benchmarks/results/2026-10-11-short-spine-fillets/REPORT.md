# Short-spine fillets: validation

Parent: `73057e9a`. The regression fails on the parent with `OutsideNeighbour` for
a radius-0.5 round along a length-0.2 edge. Its transverse supporting faces have
ample room; the old center check wrongly required the full rolling ball to fit
between the two terminal planes.

Single-edge planar terminations now validate the trimmed rational patch's positive
control hull against every original convex half-space. Center containment and the
spherical/miter corner checks remain. No public API, diagnostic meaning, dependency
or unsafe code changed. The original topology-collapse restrictions remain.

Validation:

- Full `cargo test -p truck-shapeops`: **310 passed, 0 failed, 1 existing ignored**,
  including the previously added spherical-rim matrix and seven doctests.
- The new integration test also passes in release mode. It covers 36 operations:
  three spine lengths, two radii, perpendicular/two oblique cap slopes, and original/
  rigidly transformed coordinates. Checks include exact radius, normals, end-plane
  bounds, independent volume, closed topology/mesh, history, unchanged inputs and STEP.
  An excessive transverse radius still rejects without mutation.
- All-targets Clippy for shapeops and benchmarks passes with `-D warnings`.
- One-worker action benchmark smoke, targeted nightly formatting and whitespace
  checks pass. No performance comparison or speedup is claimed.

`parent-regression.txt` records the failure before the production fix; final release,
full-suite, lint and smoke logs accompany this report. `sources.txt` records relevant
hashes. The independent geometry formulas and tolerances are in
`truck-shapeops/tests/SHORT_SPINE_FILLETS.md`.

The paired OCCT 7.9.3 probe returns valid single solids on all six unsheared native
tasks, with matching dimensions and analytic volumes (maximum integration deviation
about `1.2e-10`). Its transformed/sheared, mesh, STEP and editing outcomes were not
probed. App code is unchanged and app tests were not rerun for this batch. Experimental
corpus-capture/replay tooling was developed separately and is not part of this fix.
