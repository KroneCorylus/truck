# Native boundary distance correction

The native gradient distance query can miss a minimum on a trimmed B-spline
face. A patch cut from the target face's own support has exact directed distance
zero, but the parent reports 0.009278915241951929. A rigidly transformed copy at
half scale reports 0.004639457620975959. The candidate reports 2.33e-16 and
4.46e-16 or less respectively.

The comparison retains the original boundary query and adds tree-search surface
candidates on freeform faces. OCCT classifies each candidate against the face's
original trim before it is considered. The minimum includes original edge and
face candidates. Projectors retain stable addresses because their extrema
search refers to their internal surface adapter. Target topology is copied
without copying geometry before assembling the boundary; adding a face to a
compound otherwise changes its Free flag.

The unchanged regression fails three of 24 checks on parent 7b2604ec: two exact
surface subsets and one serialized-input preservation check. All 24 pass with
the candidate. Controls cover box/cylinder/sphere identity, separated solids in
both directions, nested-solid boundary distance, projections outside trimmed
Bezier faces, exact B-spline surface subsets, transforms/scales, and unchanged
serialized source/target inputs. Neither tests nor tolerances were weakened.
The native fixture is the existing curved-cap chamfer recorded in the fixture
README; no native reference geometry was changed.

Native volume (27 checks), reference capture (8 checks), capture harness (13
tests), and catalog (22 tests) pass. The plugin builds with warnings enabled.
All prior Rust and geometry-fixture hashes match the normal-section checkpoint:
448 shapeops, 115 stepio and 64 replay tests, all-target Clippy, and single-thread
actions smoke remain applicable. The replay is built last and its executable
hash is unchanged. The known parent pcurve_derivation precision failure and
existing ignore remain explicit; no new performance claim is made.

The full corpus must be captured from this committed implementation before
updating its counts. Sample points, sampling density, thresholds, geometry,
volume integration and operation requests are unchanged. This is a correction
to finite sampled comparisons, not a global distance bound or a claim that
Truck's remaining constructed discrepancies have been fixed.

Evidence: parent-tests.txt and parent.json, native-distance-tests.txt,
validation.json, native-volume-tests.txt, native-reference-tests.txt,
capture-tests.txt, catalog-tests.txt, plugin-build.txt, replay-build.txt.
