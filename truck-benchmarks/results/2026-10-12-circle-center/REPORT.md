# Stable circle centers near an arc endpoint

The full exact-join test run exposed an older circular-arc property failure.
Its third point lay very near an endpoint, so the Gram determinant used by
the three-point circumcenter subtracted almost equal products. The same
seed failed on the parent implementation.

Circumcenter calculation now uses the cross-product form, avoiding that
subtraction. A deterministic regression reproduces the failing arc and
requires the same center within 1e-9 for all six point orders. It fails on
parent 49ddd2fb, where the first order's center error is about 3.4e-8. The
existing property test and its retained seed now pass unchanged.

Full release geometry/modeling/meshalgo/shapeops/STEP: **870 passed**, one
unchanged PCurve derivative property failure, five existing ignored tests,
135 targets. All-target release Clippy for modeling/meshalgo/shapeops/STEP
and benchmarks and the action smoke check pass. No existing test was
deleted, ignored or loosened. Public APIs and diagnostics are unchanged;
no performance improvement is claimed.

This is a numerical construction fix, not another captured blend success.
The latest full native replay remains the clean f6871a18 capture: 72 valid
constructed outputs and 70 combined limited comparisons. The catalog keeps
that runtime evidence separate from this deterministic regression.
