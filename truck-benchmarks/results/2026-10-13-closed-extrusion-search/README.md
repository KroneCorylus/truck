# Closed extrusion endpoint inverse searches

The sampled ruling search could select the wrong endpoint of a closed profile. The two endpoints coincide in space but can have different tangents, so Newton could fail to invert an exact point or return a nearest point on the other branch. Complete rational ellipses could also invert onto an extension outside the finite profile range.

When the sampled winner is a coincident endpoint, compare local tangent projections from both endpoint branches. Clamp each candidate to its adjacent sample interval and the supplied extrusion range, and retain it only when its actual surface residual improves the seed. Newton and its existing acceptance checks still determine the result. Explicit parameter hints bypass the new seed logic. No periodicity is invented for cusp profiles.

`parent-regressions.txt` records five failures on parent `cb6e5f49`: exact cubic/quartic/rational inversion, the rational ellipse's finite branch, and a nearest point choosing the wrong side of the cusp. The explicit-hint/trial-budget regression already passes on the parent. The six new tests cover three scales, nonuniform affine transforms, reversed profiles, both seam sides, exact and off-surface queries. Existing tests and tolerances are unchanged.

The read-only Bézier-prism probe still produces an inconsistent solid and an irregular mesh; this batch does not claim a recovered STEP input or fillet. Healing, curve inclusion and boundary chart initialization remain separate work.

Validation results are recorded in `metadata.json` and the neighboring logs. No public API or dependency change, and no performance claim; the benchmark invocation is a smoke check only.

The five-crate release suite reproduces the existing `truck-geometry/tests/pcurve.rs::pcurve_derivation` precision failure: -2829663063.761872 versus -2829663063.7618704. The same parent failure is recorded in `../2026-10-13-cubic-endpoints/parent-precision-test.txt`. Its assertions remain unchanged.

Final validation: 944 passed, one unchanged precision failure, 5 existing ignored tests across 148 targets. All-target release Clippy for five affected crates plus benchmarks, action benchmark smoke, and the official replay build passed. Source hashes remained unchanged throughout validation.
