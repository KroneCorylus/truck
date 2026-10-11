# Tangent imported boundaries and full-cylinder support bounds

Native OCCT `blend/simple/Z1` exports an input containing distinct STEP vertex
identifiers at exactly the same point, leaving a one-edge circle topologically
open. Preprocessing used to miss that circle and panic on an odd split-crossing
list. After joining identical adjacent wire ends, the cap's tangent outer/inner
loops need a separating material edge to become simple faces. Generated planar
and cylindrical-generator pcurves also need conversion to modeling lines.

The repair aliases only identical endpoints that must connect in a face wire;
unrelated coincident vertices and nonzero gaps are preserved. Touching boundary
loops use a deterministic shortest unshared-vertex divisor through the existing
intersection/splitting path. Odd crossing lists return without indexing past the
end, and rejected generated curves do not partially append the polyline cache.
Cylindrical generators allow only numerical angular roundoff, additionally bounded
by the resulting chord error. Other curved pcurves are still rejected.

Full-circle STEP boundaries exposed a separate domain issue: their single-span
rational representation has nonpositive weights, so positive control-hull bounds
were unavailable. Affine circles/ellipses now use a conservative enclosing
parallelogram for domain fitting. This preserves exact analytic supports and
covers complete circles without relying on that rational conversion.

Validation:

- Parent logs reproduce the native Z1 panic, rejected linear pcurves, plain
  cylinder inconsistency, odd-divisor panic and partial cache mutation.
- Z1 passes both preprocessing modes before/after rigid motion, with consistent
  geometry, genus zero, closed meshes and independent volume 2500*pi. A second
  STEP round trip passes the same checks in all four cases.
- The resource cylinder with unsplit circular boundaries passes geometry,
  bounded support parameters, closed meshes, independent volume 40*pi and STEP.
- Pcurve tests check exact positions and first/second derivatives, orientation,
  rigid transforms, numerical seam offsets and rejection of curved helices.
- Unit checks retain unrelated coincident vertices and nonzero gaps and preserve
  the polyline cache after invalid crossings or failed curve conversion.
- Full release shapeops/STEP: **408 passed, 0 failed, 1 existing ignored**.
  All-target Clippy, one-worker action benchmark smoke, formatting and whitespace
  checks pass for source changes. Native STEP fixture whitespace is retained
  byte for byte; log terminal blank lines are normalized. No latency claim is made.

The fixture matches the archived native input byte for byte (SHA-256
`6af575731bc5a43dbcbfb0190d0e4932cdc12bf94b560398922606aeef02c224`).
This fixes preprocessing; it does not yet establish that the subsequent Z1
fillet succeeds or matches native geometry. No API, dependency, diagnostic-code
or app change is made. App tests were not rerun.
