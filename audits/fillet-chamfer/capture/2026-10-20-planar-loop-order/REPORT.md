# Planar loop order: clean full capture

Runtime d198a51cc1d1d4c7d7df2a92a4520e2673368800. All 396 native entries pass, yielding 648 operations with unchanged requests, scopes and native result properties. All 420 prior constructions retain checks; 420 complete result/comparison records are exactly unchanged. Numerical changes: 0.

438 constructions (+18); 408 limited sampled passes (+18) at 0.001. 30 constructed discrepancies remain. Every construction preserves input, consistent geometry, closed mesh, STEP preparation and valid single-solid native/Truck comparison shapes.
Statuses: {"input-import-failed": 96, "operation-succeeded": 438, "operation-failed": 99, "unhandled-intent": 6, "input-inconsistent": 9}.
Status changes:
- chamfer/dist_angle_sequence/A1/006: operation-failed → operation-succeeded
- chamfer/dist_angle_sequence/A1/008: operation-failed → operation-succeeded
- chamfer/dist_angle_sequence/A1/011: operation-failed → operation-succeeded
- chamfer/dist_angle_sequence/A1/013: operation-failed → operation-succeeded
- chamfer/dist_angle_sequence/A1/014: operation-failed → operation-succeeded
- chamfer/dist_angle_sequence/A4/002: operation-failed → operation-succeeded
- chamfer/dist_dist_sequence/A1/006: operation-failed → operation-succeeded
- chamfer/dist_dist_sequence/A1/008: operation-failed → operation-succeeded
- chamfer/dist_dist_sequence/A1/011: operation-failed → operation-succeeded
- chamfer/dist_dist_sequence/A1/013: operation-failed → operation-succeeded
- chamfer/dist_dist_sequence/A1/014: operation-failed → operation-succeeded
- chamfer/dist_dist_sequence/A4/002: operation-failed → operation-succeeded
- chamfer/equal_dist_sequence/A1/006: operation-failed → operation-succeeded
- chamfer/equal_dist_sequence/A1/008: operation-failed → operation-succeeded
- chamfer/equal_dist_sequence/A1/011: operation-failed → operation-succeeded
- chamfer/equal_dist_sequence/A1/013: operation-failed → operation-succeeded
- chamfer/equal_dist_sequence/A1/014: operation-failed → operation-succeeded
- chamfer/equal_dist_sequence/A4/002: operation-failed → operation-succeeded

Planar blend checks now locate the outer loop by projected area, allowing imported faces to list holes first. Trim validation keeps the original outer-loop identity. Polygon orientation, intersections, hole containment and spacing checks remain unchanged; no tolerance is relaxed.

Five regressions fail on the parent: three boundary/patch tests and two solid tests. Two new collision guards and three existing planar tests pass there. All seven new tests now pass. Boundary tests cover all six loop orders with transforms; sixteen solid variants check outer and inner chamfers, independent material, genus 2, closed meshes, STEP, input preservation and face correspondence. Four native controls match independent material and contact geometry. Both native inner-rim operations mutate input; that evidence is retained.

420 shapeops tests (one existing ignore), 59 replay tests, 13 capture tests, all-target release Clippy and single-thread actions smoke pass. Replay built last. Unchanged mesher 102-test evidence is verified by source hashes.

Proof:truck-benchmarks/results/2026-10-20-planar-loop-order/REPORT.md.
Archive: 8365 byte-verified members;SHA256 d16e6760497f0fc309ce1fdf28f99df686b6de48dfc5d9c865a82f806b8f1fdb.
Finite samples do not establish a global error bound or full parity. No consumer, dependency or tolerance change, push, PR or performance claim.
