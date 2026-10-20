# Mixed collapsed contacts: clean full capture

Runtime 3bd4ef6da5ff3c058c071d80dd6e92b8f039f04f. All 396 native entries pass; 648 operations retain requests, scopes and native properties. All 443 previous constructions retain checks; 443 complete result/comparison records are exactly unchanged. Numerical changes: 0. Unchanged-status failure diagnostic changes: 0.

443 constructions (+0); 413 limited sampled passes (+0) at 0.001. 30 constructed discrepancies remain.
Statuses: {"input-import-failed": 96, "operation-succeeded": 443, "operation-failed": 94, "unhandled-intent": 6, "input-inconsistent": 9}.
- No status changes.

A tangent rim can now combine collapsed equal-radius spherical corners with regular toroidal corners on larger convex cylinders. The existing helper keeps circular cap contacts for the regular corners, shares cross-sections and preserves original supports. Fold, trimming and topology validation remain in place.

Two success contracts fail on the parent; the regular/fold guard passes. Three new tests cover 20 order/transform variants and four regular/fold controls, checking material, STEP, history, input preservation, patch radii and normals. Two fresh native controls establish independent material; the new radius-1 result has material error below 5.1e-12 and maximum sampled distance below 1.1e-14, with 648 new-surface samples and contact checks.

All 443 complete prior result/comparison records are identical; 0 numerical changes are listed in the comparison summary. Existing native-policy discrepancies remain documented separately. Finite samples do not establish a global error bound.

438 shapeops tests (one existing ignore), 115 stepio tests, 59 replay tests, 13 capture tests, all-target release Clippy and single-thread actions smoke pass. Replay built last. Unchanged geometry/modeling/meshalgo source hashes retain the preceding checkpoint’s 289/91/102 passing tests and four existing ignores. Geometry’s independently reproduced parent pcurve_derivation precision failure remains unresolved under the unchanged absolute 1e-6 assertion; no test was skipped or weakened.

Proof: truck-benchmarks/results/2026-10-20-mixed-collapsed-contacts/REPORT.md.
Archive: 8380 byte-verified members; SHA256 dcf3581b77fb33ad1822592e148cf48c69a2d5ba522ed4d347aa2a0b63f991c2.
Finite samples do not establish a global bound or full parity. No consumer/dependency/tolerance change, push, PR or performance claim.
