# Parallel cylinder contact angles: clean full capture

Runtime 68b6a6425d9e352631b1e5bbfddb9080684dd7a3. All396 native entries pass, yielding648 operation records with unchanged requests, complete selection scopes and native result properties. All364 previous constructions retain successful checks; 364 complete result/comparison records are exactly unchanged, with 0 numerical changes.

365 constructions (+1); 341 limited sampled passes (+0) at0.001. 24 constructed discrepancies remain. Every construction preserves input, consistent geometry, a closed mesh, STEP preparation and valid single-solid native/Truck comparison shapes.
The new dist_angle/A2 construction preserves the requested chord and 30-degree contact angle at all six selected edges. Independent measurements of native output give 41.4783409545, 30, 46.7813237992, 30, 30 and 44.8956155730 degrees in selection order. The two cylinder/cylinder contacts agree; three plane/cylinder contacts explain the new raw discrepancy. See contact-angle-checks.json and the retained surface oracle. No comparison verdict is overridden.

Statuses: {"input-import-failed": 96, "operation-succeeded": 365, "operation-failed": 172, "unhandled-intent": 6, "input-inconsistent": 9}.
Status changes:
- chamfer/dist_angle/A2/001: unhandled-intent → operation-succeeded

Replay now resolves cylinder/cylinder distance-angle requests from the reference chord and angle at the new contact. It intersects the resulting ray with the other cylinder, selects the first forward intersection, and rejects missing/out-of-branch contacts. Core geometry and existing plane/cylinder interpretation are unchanged.

Two new tests fail on parent9b69102c; six tests pass. The former blanket rejection was rewritten on that parent into an impossible-union guard, preserving input checks. Twenty positive variants cover equal/unequal radii, both references, feasible30/45-degree requests, transforms and reversals. All57 release replay tests,13capture tests, Clippy and benchmark smoke pass; replay built last. Core sources/tests exactly match the prior400shapeops/98meshalgo validation (one existing ignore each); results are reused with hashes, not claimed rerun.

Twelve native controls: ten valid constructions agree with the independent two-circle contact/material oracle and Truck samples; two union45-degree requests fail natively and are rejected by Truck. Their negative quadratic discriminants independently demonstrate no intersection. Native spline-backed bevels are checked against the expected plane at81 surface points. Eight successful native inputs stay unchanged; two union inputs mutate endpoint tolerances. Raw failures and mutations remain visible; Truck inputs stay unchanged.

Proof: truck-benchmarks/results/2026-10-20-parallel-cylinder-angles/REPORT.md.
Archive: 8146 byte-verified members; SHA256 59b1bc4431dec81a4894f86b628ac8cb84878d265eadce97a06806568836fa60.
Finite samples do not establish global boundary error or full parity. No consumer change, push, PR or performance claim.
