# Revolution-angle precision

The captured spherical-sector input from `blend/simple/D2` was geometrically valid
in OCCT but failed Truck's curve-in-surface check. Computing a revolution angle
with inverse cosine amplified floating-point error near coincident/opposite
meridians. Signed `atan2` of the projected cross and dot products preserves small
angular offsets and orientation. No tolerance, public API or diagnostic changed.

Two regressions fail with the parent implementation and pass with the fix:

- Point and nearest-point searches at three radii, six initial azimuths and eight
  seam/antipodal angles recover points within 1e-9.
- Exact spherical meridian curves remain included at radii 1, 150 and 1000, three
  azimuths, original/shifted angular domains, rigid transforms and reversed
  orientations. Off-sphere curves remain rejected.

The original captured D2 input now passes geometric consistency in a separate
inspection. This does not establish success of its subsequent blend.

Validation: full geometry/modeling/shapeops/STEP suites: **728 passed,
0 failed, 4 existing ignored**, including doctests. All-targets Clippy for
those four crates and benchmarks passes with `-D warnings`; one-worker action
benchmark smoke and targeted nightly formatting pass. No speedup or comparative
performance claim is made. Parent failures, final logs and source hashes accompany
this report. App code is unchanged; app tests were not rerun.

The capture/replay tooling was committed separately in `1a423b92`. A new failing
cone STEP-domain regression was developed after the full suite was launched; it
is intentionally excluded from this fix and its totals. Its production fix is
separate work.
