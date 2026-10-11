# Curved fillet end domains in STEP

The captured OCCT `blend/simple/C7` task revealed a real export loss. Its curved
conical termination rises beyond the straight contacts' endpoints. The procedural
surface evaluates that region correctly, but STEP fitting previously covered only
its nominal longitudinal range. Native STEP import then lost about 540.48 volume
units, despite reporting a valid single solid.

STEP preparation now normalizes each procedural fillet, measures the prepared
spatial boundary curves in its local parameter system, and fits the range actually
used by those curves. Expanded ends receive one sample-step margin. World-space
residuals and finite parameters are checked; original topology, geometry and
fitting tolerance are retained. The method remains sampled, not a certified global
bound. No public API, diagnostic meaning, dependency or unsafe code changed.

Validation:

- The final four-operation matrix fails on the parent with a prepared-surface
  boundary discrepancy of 0.00487 versus the required 0.0001. Two radii, native and
  rigidly transformed/reversed selections check exact cylindrical radius/normals,
  independent volume, geometric consistency, topology, closed meshes, history,
  immutable inputs, finite exported-domain coverage and STEP round trips.
- The native-size prototype also fails on the parent and passes with the fix.
  An initial matrix run had the expected normal reversed in the test fixture;
  that expectation was corrected before the final matrix and parent run. Its log
  is retained and is not classified as a production failure.
- The 13 existing targeted conical/spherical, representation, parameter-domain,
  constant/variable-chain, history and export tests pass in release mode.
- Full release shapeops and STEP suites: **391 passed, 0 failed, 1 existing ignored**,
  including doctests. The full suites were repeated after the final finite-bound
  guard. Final all-targets Clippy passes with `-D warnings`; one-worker action
  benchmark smoke, targeted nightly formatting and whitespace checks pass.
- A fresh captured C7 replay with source/binary hashes preserves input and produces
  a consistent solid with a closed Truck mesh. Native OCCT accepts its STEP as a
  valid single solid. Bidirectional sampled boundary residuals are 1.77e-6 and
  2.76e-6, versus 4.2283 in the old export. STEP volume is 76158.86323932; native
  volume is 76158.86301431 and the independent integral is 76158.86301257.

The final native comparison, Truck output and STEP accompany this report. Their
STEP-preparation source hash matches `sources.txt`; the complete instrumented
capture is retained locally and will be attached to the audit separately. These
checks do not certify global intersections, downstream edits or all possible trim
extrema. No comparative latency claim is made. App code is unchanged and app tests
were not rerun.

The formulas and physical task are documented in
`truck-shapeops/tests/CURVED_FILLET_STEP_DOMAIN.md`. A separate new transformed-curve
sampling regression was developed during validation and is not included in this
batch or its totals.
