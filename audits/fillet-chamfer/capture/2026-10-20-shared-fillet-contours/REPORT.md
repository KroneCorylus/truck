# Shared and open fillet contours: clean full capture

Runtime f474ab2639adf1418b6faab53efb8c176c333d2f. All 396 native entries pass, yielding 648 operations with unchanged requests, scopes and native result properties. All 441 prior constructions retain checks; 441 complete result/comparison records are exactly unchanged. Numerical changes: 0.

442 constructions (+1); 412 limited sampled passes (+1) at 0.001. 30 constructed discrepancies remain. Every construction preserves input, consistent geometry, closed mesh, STEP preparation and valid single-solid native/Truck comparison shapes.
Statuses: {"input-import-failed": 96, "operation-succeeded": 442, "operation-failed": 95, "unhandled-intent": 6, "input-inconsistent": 9}.
Status changes:
- blend/simple/I6/001: operation-failed → operation-succeeded

Separate fillet contours can now be open or closed and share original support faces. Each tangent component uses the existing wire constructor. Remaining selected edges must survive, and a later component cannot modify an earlier generated fillet. Failure preserves the input, and final face correspondence refers to the original solid.

Two new rim tests and the deliberately rewritten shared-cone success test fail on the parent. All six focused tests pass after implementation, including the two unchanged existing contour tests and the new overlap guard. Eight positive and eight overlapping-rim variants cover subdivisions, transforms and reversed ordering. Three fresh native controls verify independent material, contact positions and rolling radius; 810 sampled surface points and valid single-solid STEP checks are retained. Native I6 also passes the targeted comparison.

431 shapeops tests (one existing ignore), 59 replay tests, 13 capture tests, all-target release Clippy and single-thread actions smoke pass. Replay built last. Unchanged mesher 102-test evidence is verified by source hashes.

Proof: truck-benchmarks/results/2026-10-20-shared-fillet-contours/REPORT.md.
Archive: 8377 byte-verified members; SHA256 372c490c73f87321b402ede3d12c75f356809ee9394ab55b92fb8f5dbc8243f8.
Finite samples do not establish a global error bound or full parity. No consumer, dependency or tolerance change, push, PR or performance claim.
