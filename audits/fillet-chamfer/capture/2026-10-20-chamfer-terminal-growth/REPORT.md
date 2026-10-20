# Chamfer terminal growth: clean full capture

Runtime 5eed074bc467f2b527a19a578fb97e81a5920257. All 396 native entries pass, yielding 648 operations with unchanged requests, scopes and native result properties. All 417 prior constructions retain checks; 417 complete result/comparison records are exactly unchanged. Numerical changes: 0.

420 constructions (+3); 390 limited sampled passes (+3) at 0.001. 30 constructed discrepancies remain. Every construction preserves input, consistent geometry, closed mesh, STEP preparation and valid single-solid native/Truck comparison shapes.
Statuses: {"input-import-failed": 96, "operation-succeeded": 420, "operation-failed": 117, "unhandled-intent": 6, "input-inconsistent": 9}.
Status changes:
- chamfer/dist_angle/F4/001: operation-failed → operation-succeeded
- chamfer/dist_dist/F4/001: operation-failed → operation-succeeded
- chamfer/equal_dist/F4/001: operation-failed → operation-succeeded

Modeling chamfers now reuse exact straight-edge extension at open wire ends. This lets the contact move inward at a concave radial step. A private trim callback leaves public generic bounds unchanged. Closed and interior edges retain bounded trims; final geometry, topology and boundary checks remain.

Three regressions fail on the parent; convex and oversized guards pass. All five now pass: twelve positive variants check two sector shapes, three setback pairs, transformed/reversed inputs, independent volume, contact circles, closed meshes, STEP and face correspondence. Eight oversized variants reject without mutation. Six fresh native controls match independent material; all six Truck controls preserve input and pass the limited native comparison. Three stepped native controls mutate vertex tolerances; their before/after evidence is retained.

413 shapeops tests (one existing ignore), 59 replay tests, 13 capture tests, all-target release Clippy and single-thread actions smoke pass. Replay built last. Unchanged mesher 102-test evidence is verified by source hashes.

Proof:truck-benchmarks/results/2026-10-20-chamfer-terminal-growth/REPORT.md.
Archive: 8311 byte-verified members;SHA256 162e3eb38895b5ee7028636c86c0484a3840788ca4109782b083a24c907d6709.
Finite samples do not establish a global error bound or full parity. No consumer, dependency or tolerance change, push, PR or performance claim.
