# Exact nonelliptic conic import

Finite STEP parabolas and hyperbolas now convert into exact quadratic polynomial
and positive-weight rational curves, respectively, on normalized [0, 1] domains.
Hyperbolas split into equal spans no wider than two native parameter units. The
weighted tangent-intersection construction preserves the conic exactly, including
reversed orientation, affine transforms and swept surfaces. Non-finite geometry
and invalid trim intervals return conversion errors.

Native F8 replay also exposed a separate parser defect: both 2D and 3D parabola
edges were constructed as hyperbolas. The parser now uses the correct conic, tries
its analytic point inverse before nearest-point fallback, and treats hyperbolic
intervals as nonperiodic. The previous 3D path added 2π to short hyperbola trims.

Validation:

- The final parent run fails the prism, both modeling conversion and both STEP
  edge-parser regressions; the existing invalid-input rejection still passes.
- The conversion matrix covers 24 transformed/reversed parabolic and hyperbolic
  arcs, 129 forward and independent reverse samples, endpoints, oriented tangents,
  rescaled polynomial derivatives, positive rational weights, tangent continuity,
  and swept-surface geometry/normals.
- Sixteen actual STEP edge fixtures cover both dimensions, both senses, both conic
  types and ordinary/tiny parameter spans. Four skew-extruded prism variants check
  consistent geometry, closed topology/meshes, independent analytic volume and STEP.
- Full release STEP and shapeops suites: **397 passed, 0 failed, 1 existing ignored**.
  All-target Clippy, one-worker action benchmark smoke, nightly formatting and
  whitespace checks pass. No comparative latency claim is made.
- Fresh instrumented native F8/F9/G1/G2 remain successful. Truck now imports their
  conics but rejects inconsistent input support domains before blending. No Truck
  blend success or full parity is claimed. Exact replay files and source hashes are
  retained in `target/occt-capture-conic-parser-fixed` for attachment to the audit.

During test development, using the old hyperbola nearest-point implementation as
an oracle exposed its independent failure on an exact point. The final geometric
checks use the analytic point-on-curve inverse and retain their world residual
assertions. Tiny trim tests also required normalized output knots rather than
preserving the source parameter scale. Neither changes the requested geometry.
No existing tests, public signatures, diagnostic codes, dependencies or unsafe
code changed. App code is unchanged and its tests were not rerun.
