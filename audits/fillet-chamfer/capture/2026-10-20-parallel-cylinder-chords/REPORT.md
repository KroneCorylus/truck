# Parallel cylinder chords: clean full capture

Runtime 7ab0f68f4641832a590fb25abb1c419f1669955b. All 396 native entries pass, yielding 648 operations with unchanged requests, complete selection scopes and native result properties. All 363 previous constructions retain their successful checks; 362 complete result/comparison records are exactly unchanged; 1 numerical changes are recorded.

364 constructions (+1); 341 limited sampled passes (+1) at 0.001. 23 constructed discrepancies remain. Every construction preserves its input, consistent geometry, a closed mesh, STEP preparation and valid single-solid native/Truck comparison shapes.

The prior equal_dist/A2 result also improves: correcting both cylinder chords reduces the bidirectional sampled discrepancy from about 0.00062106 to below 4.6e-13. Its material values change accordingly; all prior successful checks remain satisfied.

Statuses: {"input-import-failed": 96, "operation-succeeded": 364, "operation-failed": 172, "unhandled-intent": 7, "input-inconsistent": 9}.
Status changes:
- chamfer/dist_dist/A2/001: unhandled-intent → operation-succeeded

Replay converts the chord on each parallel cylindrical support to its core tangent setback, preserving equal/unequal distances and reference assignment. The core now constructs the extra planar cap region for concave chamfers across coplanar subdivided end caps while preserving existing internal seams. No selected edge is dropped. Two-cylinder distance-angle remains explicitly unsupported.

Three parent failures and two passing rejection guards precede implementation. All five tests now pass, including 18 replay variants with transforms, reversed references and face/sample order, exact contact/material and shared solid/STEP checks. Full release validation passes 400 shapeops tests, 98 meshalgo tests (one existing ignore each), 54 replay tests and 13 capture tests. Clippy and benchmark smoke pass; replay built last.

Nine independent native/Truck controls match exact contact and material oracles and sampled geometry. Truck inputs remain unchanged. Native changes endpoint tolerances in three union controls; the original failed capture assertion and raw before/after BREPs remain in the evidence. This is not claimed as native input immutability.

Proof: truck-benchmarks/results/2026-10-20-parallel-cylinder-chords/REPORT.md.
Archive: 8143 byte-verified members; SHA256 a5de09d82068bcee7ac0f9e60e66b7f819b69913823ce7e64e6f0007c1350423.
Finite samples do not prove global boundary error or full parity. No consumer change, push, PR or performance claim.
