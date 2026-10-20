# Cylindrical terminal fans: clean full capture

Runtime ea20f7d957f223b8f5b370ddd2089cc1ede832aa. All 396 native entries pass, yielding 648 operations with unchanged requests, complete selection scopes and native result properties. All 365 previous constructions retain their successful checks; 351 complete result/comparison records are exactly unchanged. Numerical changes: 14 records, of which 0 change STEP material or sampled-boundary measurements rather than mesh volume alone. Every change is retained in comparison-summary.json.

371 constructions (+6); 347 limited sampled passes (+6) at 0.001. 24 constructed discrepancies remain. Every construction preserves input, consistent geometry, a closed mesh, STEP preparation and valid single-solid native/Truck comparison shapes.
Statuses: {"input-import-failed": 96, "operation-succeeded": 371, "operation-failed": 166, "unhandled-intent": 6, "input-inconsistent": 9}.
Status changes:
- chamfer/dist_angle_complex/C4/001: operation-failed → operation-succeeded
- chamfer/dist_angle_sequence/C4/002: operation-failed → operation-succeeded
- chamfer/dist_dist_complex/C4/001: operation-failed → operation-succeeded
- chamfer/dist_dist_sequence/C4/002: operation-failed → operation-succeeded
- chamfer/equal_dist_complex/C4/001: operation-failed → operation-succeeded
- chamfer/equal_dist_sequence/C4/002: operation-failed → operation-succeeded

Concave chamfers now preserve internal seams across coincident cylindrical terminal faces and add the missing exact cylindrical patch. Coincidence, orientation, local visible-half and projected-polygon checks guard the branch. The mesh refinement cap changes from 20 to 64 rounds, retaining early exit; exploratory sampling changes were discarded.

Two construction regressions fail on parent 3ce55545. Four independent normal/robust mesh regressions also fail with the parent mesher. Twenty-four construction variants cover equal, unequal and angle requests, one/two edges, explicit cylinder subdivisions, transforms and reversals. Full release validation passes 400 shapeops tests, 102 meshalgo tests (one existing ignore each), 59 replay tests, 13 capture tests, all-target Clippy and benchmark smoke. Replay built last.

All twelve Truck controls match analytic material/contacts and the equivalent valid native controls. Six explicitly subdivided native controls remain raw discrepancies: all mutate input tolerances, three report invalid output, and the other three have wrong material with stored tolerances around 5 units. Six equivalent unsplit native controls are valid, immutable and match the independent section integral. Raw and equivalent-reference results remain distinct; no raw verdict is overridden.

Proof: truck-benchmarks/results/2026-10-20-cylindrical-terminal-fans/REPORT.md.
Archive: 8164 byte-verified members; SHA256 8c8eff56231a82545b4c919910615e89e59e9d2d47c207fa909b1a5cba47bbd8.
Finite samples do not establish global error bounds or full parity. No consumer change, push, PR or performance claim.
