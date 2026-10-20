# Tilted-cap chamfers: clean full capture

Runtime fdc8092c2cd5932b9c3ce93a3cd150be32134741. All 396 native entries pass; 648 operations retain requests, scopes and native properties. All 443 previous constructions retain checks; 443 complete prior records are exactly unchanged. Numerical changes: 0. Unchanged-status failure diagnostic changes: 0.

444 constructions (+1); 414 limited sampled passes (+1) at 0.001. 30 constructed discrepancies remain.
Statuses: {"input-import-failed": 96, "operation-succeeded": 444, "operation-failed": 93, "unhandled-intent": 6, "input-inconsistent": 9}.
- chamfer/equal_dist/E5/001: operation-failed → operation-succeeded

Tangent chamfer contacts can now continue to support seams beneath a tilted planar cap with parallel convex cylindrical and planar walls. The original construction runs first; this recovery is restricted to contact-trimming failures in that analytic scope. Contacts stay on original supports, folds remain rejected, and endpoint corrections stay within caller tolerance.

Two success contracts fail on the parent; two controls pass. Four new tests cover both directions, positive and negative tilt, similarity transforms, setbacks, material, history, STEP and immutable inputs. Four fresh native controls preserve their inputs and produce valid single solids. Three meet the unchanged 0.001 sampled-distance threshold. The 10° control fails it at 0.002511: an independently extracted native cap-boundary point is at most 0.096315 from the original guide for requested setback 0.1. Truck contact samples have maximum error 2.051e-6. The discrepancy and failed comparison remain recorded; its cause and global error bounds are unresolved.

442 shapeops tests (one existing ignore), 115 stepio tests, 59 replay tests, 13 capture tests, all-target release Clippy and single-thread actions smoke pass. Full suites preceded a mechanical iterator cleanup; seven affected integration targets and replay tests passed afterward, with the exact patch and both source hashes retained. Replay built last. Unchanged geometry/modeling/meshalgo hashes retain 289/91/102 passing tests and four existing ignores. The independently reproduced parent pcurve_derivation precision failure remains unresolved under the unchanged absolute 1e-6 assertion. No test was weakened or skipped.

Proof: truck-benchmarks/results/2026-10-20-tilted-cap-chamfers/REPORT.md.
Archive: 8383 byte-verified members; SHA256 c64b7fa26745fb30bc0e46a12270cee0f9989d5ae51cb980892c2f9aa7519356.
Finite samples do not establish a global bound or full parity. No consumer/dependency/tolerance change, push, PR or performance claim.
