# Convex chamfer corner clipping: clean full capture

Runtime 070d198568a9d8469a666ed330931fde90473fe2. All 396 native entries pass, yielding 648 operations with unchanged requests, complete selection scopes and native result properties. All 371 previous constructions retain successful checks; 371 complete result/comparison records are exactly unchanged. Numerical changes: 0 records, retained in comparison-summary.json.

372 constructions (+1); 347 limited sampled passes (+0) at 0.001. 25 constructed discrepancies remain. Every construction preserves input, consistent geometry, a closed mesh, STEP preparation and valid single-solid native/Truck comparison shapes.
Statuses: {"input-import-failed": 96, "operation-succeeded": 372, "operation-failed": 165, "unhandled-intent": 6, "input-inconsistent": 9}.
Status changes:
- chamfer/dist_angle_complex/A1/001: operation-failed → operation-succeeded

Three selected convex bevels whose miter escapes an original support plane now terminate on that plane. The dominant bevel has two end tips, shared by its neighbors and the original face. All new tips must satisfy local original and bevel halfspaces. Other miter branches and polygon checks remain intact.

Four cases fail on parent 002c17ac; the balanced-corner guard passes there. Five tests, each with identity and transformed/reordered variants, check exact vertices, independent convex-halfspace material, topology, closed mesh, geometric consistency, shared STEP checks, generated-face count and input immutability. Five native replay controls retain raw differences between native corner transitions and Truck's planar miters. All Truck controls match the independent halfspace oracle. No native discrepancy is relabeled as a match.

Full release validation:405 shapeops tests (one existing ignore),59 replay tests,13 capture tests,all-target Clippy and single-thread benchmark smoke. Replay built last. The unchanged mesher's102-test evidence is reused with verified source hashes.

Proof: truck-benchmarks/results/2026-10-20-convex-corner-clipping/REPORT.md.
Archive:8167 byte-verified members; SHA256 7ee4626dab7f8e2676b36ed4ea27cb441295eb71cb24b1a4f84eab8a2f0566b9.
Finite samples do not prove global error bounds or full parity. No consumer change, push, PR, tolerance change or performance claim.
