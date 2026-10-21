# Unequal tilted-cap chamfers: clean full capture

Runtime 51e2c1eca1e9be15cf2f7ada067a6323ff39823d. All 396 native entries pass; 648 operations retain requests, scopes and native properties. All 444 previous constructions retain checks; 444 complete prior records are exactly unchanged. Numerical changes: 0. Unchanged-status failure diagnostic changes: 0.

445 constructions (+1); 415 limited sampled passes (+1) at 0.001. 30 constructed discrepancies remain.
Statuses: {"input-import-failed": 96, "operation-succeeded": 445, "operation-failed": 93, "unhandled-intent": 5, "input-inconsistent": 9}.
- chamfer/dist_dist/E5/001: unhandled-intent → operation-succeeded

Replay now accepts unequal setbacks on tilted plane/cylinder rims while preserving the captured reference side across the full native contour. The production change is three lines in replay; kernel geometry is unchanged. Tilted distance-angle requests remain unsupported because they need a varying local-angle construction.

One success contract fails on the parent; horizontal and angle-rejection controls pass. Three tests cover 24 tilted and 8 horizontal reference/order combinations, material, immutable inputs and STEP. Exported geometry checks 6,336 cap/wall contact points. Eight fresh native controls produce valid solids: five meet the unchanged 0.001 distance threshold, three fail at 0.001292, 0.006632 and 0.001599. Independent guide witnesses prove native cap-contact shortfalls above 0.001 for all three. Their causes and global error bounds remain unresolved; failed comparisons are retained.

62 release replay tests, 13 capture tests, all-target release Clippy and single-thread actions smoke pass. Replay built last. Unchanged core hashes retain 442 shapeops and 115 stepio checks, including the previous mechanical iterator cleanup and focused retests, plus 289/91/102 geometry/modeling/meshalgo passes. The independently reproduced parent pcurve_derivation precision failure and existing ignores remain explicit. No existing test was weakened or skipped.

Proof: truck-benchmarks/results/2026-10-20-tilted-cap-unequal/REPORT.md.
Archive: 8386 byte-verified members; SHA256 8f09c688f0ae2db460ff39b39c9d293a5af8277512d16a65c1f2e0ada3c4dec3.
Finite samples do not establish a global bound or full parity. No consumer/dependency/tolerance change, push, PR or performance claim.
