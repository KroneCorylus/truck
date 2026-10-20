# Cylinder-reference longitudinal angles: clean full capture

Runtime 460ac6fd23a92e82a4f94f92942acbd02a2abbac: all 396 native entries pass, producing 648 operation records.
Requests, complete scopes and native result properties are unchanged.
All 359 prior constructions retain their successful checks; 359 complete prior
result/comparison records are exactly unchanged and 0 numerical changes
are recorded. No prior successful check regressed.
All four newly constructed corpus outputs preserve the requested 30-degree
contact angle; their native references measure 41.4783409545 degrees. Both retain
the requested chord. See contact-angle-checks.json and the independent plane oracle.

363 constructions (+4); 340 limited sampled passes
at 0.001 (+0); 23 constructed discrepancies.
All constructed outputs preserve input and pass consistency, closed mesh, STEP
preparation and native valid-single-solid checks.
Statuses: {"input-import-failed": 96, "operation-succeeded": 363, "operation-failed": 172, "unhandled-intent": 8, "input-inconsistent": 9}.
Status changes:
- chamfer/dist_angle_complex/A6/001: unhandled-intent → operation-failed
- chamfer/dist_angle_complex/A8/001: unhandled-intent → operation-failed
- chamfer/dist_angle_sequence/A6/005: unhandled-intent → operation-succeeded
- chamfer/dist_angle_sequence/A6/006: unhandled-intent → operation-succeeded
- chamfer/dist_angle_sequence/A8/005: unhandled-intent → operation-succeeded
- chamfer/dist_angle_sequence/A8/006: unhandled-intent → operation-succeeded

Cylinder-reference longitudinal requests now preserve their captured chord and
the angle between the bevel and cylinder tangent at the new contact. Three
parent-failing tests cover 20 variants; a prior unsupported valid request was
replaced by positive coverage and an oversized guard on the parent first.
All 49 replay tests, 13 capture tests, Clippy, benchmark smoke and final replay
build pass. Core sources/tests are byte-identical to the previous 400 shapeops /
98 meshalgo validation; that validation is reused with hashes, not claimed rerun.

Ten independent controls verify chord, contact angle, actual contact vertices,
analytic material and valid STEP. Four native outputs agree; six native outputs
do not preserve this contact-angle contract. A separate model of the pinned
native routine reproduces its planar setbacks. Raw differences are retained.
Proof: truck-benchmarks/results/2026-10-20-longitudinal-cylinder-angles/REPORT.md.
Archive: 8140 byte-verified members; SHA256 e7fd527b6ee8964ad511b27fc12229e36ace8fcc973c50c845a5e7d63e0b33a0.
Finite samples do not prove a global error bound or full parity.
No consumer changes, push, PR or performance claim.
