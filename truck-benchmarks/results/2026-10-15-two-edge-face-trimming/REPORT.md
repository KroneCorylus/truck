# Shared-neighbor face trimming

A half-cylinder top bounded by a semicircle and one chord made both
`simple_chamfer` and `simple_fillet` panic with “This wire is not closed”.
Their shared trimming routine independently cut the same chord twice, then
wrote three replacement positions into a two-edge boundary.

The routine now applies the second cut to the already trimmed neighbor and
retains the middle segment. Boundary replacement uses edge identity, preserving
order and face orientation for both shared and distinct neighbors. Invalid
resulting face boundaries return `None` through `Face::try_new`.

Two public API regressions fail on parent `1e0c5b19` with the original panic.
The final tests cover both face encodings, swapped supports, exact curves and
shifted NURBS parameters, and rigid motion. They check immutable input,
consistent geometry, closed face boundaries, opposite shared-edge traversal,
unchanged boundary counts, the retained chord endpoints, analytic circular
contacts, and the bevel/rolling surface against independent section formulas.
These are face-patch results; this group does not claim a complete solid edit
for this half-cylinder fixture.

The initial analytic fillet check requested a 0.001 fitting tolerance while
requiring 0.000001 positional accuracy. Its failure is retained in
`initial-coarse-fillet-test.txt`. The final test requests a stricter 0.0000001
fit; the geometric assertions are unchanged. No existing test was loosened.
Final parent and corrected runs use the same final regression source.

Validation totals and source/executable hashes are recorded in metadata.json.
The affected release suites passed: 504 passed, 0 failed, 1 existing ignored.
All 15 replay tests, all-target release Clippy and action smoke passed. The
production replay was built last, after those checks. No performance claim.

The clean full native capture follows the implementation commit; its archive,
comparison, catalog and HTML checkpoint will be committed separately. Native
contour intent is still seed-only until the next capture/replay achievement.
