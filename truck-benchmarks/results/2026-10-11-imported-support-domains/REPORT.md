# Imported extrusion support domains

STEP cylinders and conic extrusions now receive finite support domains covering
all boundary curves. Positive-weight rational control hulls bound the linear sweep
coordinate, including extrema between vertices. Nonperiodic conic profiles extend
to cover the face's projected parameter range. This reparameterizes the same
analytic support; it does not relax modeling inclusion beyond a bounded surface.
Unsupported boundary representations retain their previous conversion behavior.

Cylinder-to-modeling conversion preserves the fitted axial vector. Affine conic
coordinates provide exact inclusion/search seeds for elliptical, sheared and
oblique extrusions while preserving rejection outside their trimmed domains.
Off-surface affine nearest projection retains the existing general search path;
all existing hint/projection tests remain unchanged. The STEP cylinder writer uses
perpendicular radius rather than the generator point's distance to the axis origin.

Validation:

- Parent regressions reject imported cylinder/parabolic solids, reject a curved
  trim extending beyond endpoint bounds, and incorrectly accept an out-of-domain
  affine extrusion curve. The separate exporter regression reports radius 37.054
  instead of 2 for a generator shifted by -37 along the axis.
- Eight small/long/negative/oblique cylinder and rigid-transform cases plus the
  captured native F8 parabolic prism pass geometry consistency, topology, closed
  mesh, independent analytic volume, sampled bounded-domain checks and a second
  STEP round trip. The F8 fixture is hashed and its provenance is documented.
- A parabolic trim sheet whose endpoints have zero sweep height but whose middle
  reaches 5 verifies complete curved bounds and profile parameters beyond [-1,1].
  Twelve affine conic extrusion configurations check exact point/inclusion search
  and reject wrong axial/angular bounds and off-surface curves. Three axial shifts
  verify STEP cylinder radius and sampled geometry independent of axis origin.
- Full release modeling, shapeops and STEP suites: **488 passed, 0 failed, 3 existing
  ignored**. Final all-target Clippy and one-worker action benchmark smoke pass.
  Formatting and whitespace checks pass. No latency claim is made.
- Full suites exposed two dependencies during development: off-surface affine
  projection had changed by a few ulps, and raw STEP cylinder export used the wrong
  radius after axial reparameterization. Both are fixed without weakening any
  existing test; the failing full-suite logs are retained.

A five-case prototype replay constructs A9 and F8 as consistent, immutable solids
with closed meshes and valid STEP. A9 agrees to 1.81e-13 sampled boundary distance;
F8 agrees within 8.41e-6, with STEP volume 132355676.2656 versus native
132355711.5483. F9/G1/G2 reach the blend stage. A broader 156-entry replay records
64 constructed outputs versus baseline 45, and 62 satisfy combined limited checks.
That replay predates the final raw STEP cylinder writer correction and an audit
selection-matcher fix being developed separately. Exact source/binary hashes are
retained in `target/occt-capture-support-domains-all`; these are intermediate
measurements, not full parity verdicts. Fresh final replay remains to be attached
to the audit. App code is unchanged and app tests were not rerun.
