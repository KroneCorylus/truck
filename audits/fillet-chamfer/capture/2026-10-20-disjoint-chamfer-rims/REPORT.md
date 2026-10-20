# Disjoint chamfer rims: clean full capture

Runtime edcc9bf9501c95161c05cc95c96ed33c6b3ca69a. All 396 native entries pass, yielding 648 operations with unchanged requests, complete selection scopes and native result properties. All 372 previous constructions retain successful checks; 372 complete result/comparison records are exactly unchanged. Numerical changes: 0 records, retained in comparison-summary.json.

417 constructions (+45); 387 limited sampled passes (+40) at 0.001. 30 constructed discrepancies remain. Every construction preserves input, consistent geometry, a closed mesh, STEP preparation and valid single-solid native/Truck comparison shapes.
Statuses: {"input-import-failed": 96, "operation-succeeded": 417, "operation-failed": 120, "unhandled-intent": 6, "input-inconsistent": 9}.
Status changes:
- chamfer/dist_angle/A5/001: operation-failed → operation-succeeded
- chamfer/dist_angle/A6/001: operation-failed → operation-succeeded
- chamfer/dist_angle/A8/001: operation-failed → operation-succeeded
- chamfer/dist_angle/A9/001: operation-failed → operation-succeeded
- chamfer/dist_angle/B1/001: operation-failed → operation-succeeded
- chamfer/dist_angle/B2/001: operation-failed → operation-succeeded
- chamfer/dist_angle/B3/001: operation-failed → operation-succeeded
- chamfer/dist_angle/B4/001: operation-failed → operation-succeeded
- chamfer/dist_angle/B7/001: operation-failed → operation-succeeded
- chamfer/dist_angle/B8/001: operation-failed → operation-succeeded
- chamfer/dist_angle/B9/001: operation-failed → operation-succeeded
- chamfer/dist_angle/C2/001: operation-failed → operation-succeeded
- chamfer/dist_angle/C3/001: operation-failed → operation-succeeded
- chamfer/dist_angle/C4/001: operation-failed → operation-succeeded
- chamfer/dist_angle/C5/001: operation-failed → operation-succeeded
- chamfer/dist_dist/A5/001: operation-failed → operation-succeeded
- chamfer/dist_dist/A6/001: operation-failed → operation-succeeded
- chamfer/dist_dist/A8/001: operation-failed → operation-succeeded
- chamfer/dist_dist/A9/001: operation-failed → operation-succeeded
- chamfer/dist_dist/B1/001: operation-failed → operation-succeeded
- chamfer/dist_dist/B2/001: operation-failed → operation-succeeded
- chamfer/dist_dist/B3/001: operation-failed → operation-succeeded
- chamfer/dist_dist/B4/001: operation-failed → operation-succeeded
- chamfer/dist_dist/B7/001: operation-failed → operation-succeeded
- chamfer/dist_dist/B8/001: operation-failed → operation-succeeded
- chamfer/dist_dist/B9/001: operation-failed → operation-succeeded
- chamfer/dist_dist/C2/001: operation-failed → operation-succeeded
- chamfer/dist_dist/C3/001: operation-failed → operation-succeeded
- chamfer/dist_dist/C4/001: operation-failed → operation-succeeded
- chamfer/dist_dist/C5/001: operation-failed → operation-succeeded
- chamfer/equal_dist/A5/001: operation-failed → operation-succeeded
- chamfer/equal_dist/A6/001: operation-failed → operation-succeeded
- chamfer/equal_dist/A8/001: operation-failed → operation-succeeded
- chamfer/equal_dist/A9/001: operation-failed → operation-succeeded
- chamfer/equal_dist/B1/001: operation-failed → operation-succeeded
- chamfer/equal_dist/B2/001: operation-failed → operation-succeeded
- chamfer/equal_dist/B3/001: operation-failed → operation-succeeded
- chamfer/equal_dist/B4/001: operation-failed → operation-succeeded
- chamfer/equal_dist/B7/001: operation-failed → operation-succeeded
- chamfer/equal_dist/B8/001: operation-failed → operation-succeeded
- chamfer/equal_dist/B9/001: operation-failed → operation-succeeded
- chamfer/equal_dist/C2/001: operation-failed → operation-succeeded
- chamfer/equal_dist/C3/001: operation-failed → operation-succeeded
- chamfer/equal_dist/C4/001: operation-failed → operation-succeeded
- chamfer/equal_dist/C5/001: operation-failed → operation-succeeded

Core selected-edge chamfers now handle multiple disjoint tangent wires with constant distances on each oriented side. Components follow original shell order and may share original support faces. Selected edges must survive preceding operations, and a later cut cannot modify an earlier generated bevel. Final history maps back to the original solid. The existing wire constructor handles the geometry; selection scope is not reduced and replay needs no special workaround.

Two positive tests fail on parent 69518ba1; the overlap guard passes there. Sixteen positive variants cover full/quarter cylinders, split arcs, four distance patterns and transformed/reordered inputs. Four negative variants reject overlapping cuts without mutation. Independent cone-end material, exact contact vertices, closed meshes, STEP round trips and complete face correspondence are checked.

Eight fresh native controls are valid, preserve input and match the independent cone-end formula. All eight Truck controls preserve expected material and contacts and pass the 0.001 limited native comparison. Conical geometry is checked independently using sampled surface points. These finite samples do not establish a global error bound.

Validation: 408 shapeops tests (one existing ignore), 59 replay tests, 13 capture tests, all-target Clippy and single-thread benchmark smoke. Replay built last. The unchanged mesher retains its 102-test evidence with source hashes verified.

Proof: truck-benchmarks/results/2026-10-20-disjoint-chamfer-rims/REPORT.md.
Archive: 8302 byte-verified members; SHA256 dc4e55b7ed3862779487b9daa79fc6062f38e1a8f013a06b21044e75c6ad14c7.
No consumer, dependency or tolerance change, push, PR, performance claim or full parity claim.

The five new unequal-distance discrepancies (B3, B4, C2, C3, C4) are plane/cone rims. The existing independent cone-contact oracle measures both selected rims in each result: 20 measurements. Truck preserves both requested setbacks with maximum sampled error 8.88e-9. On affected acute rims, native first setback 0.1 becomes 0.0736907400868 for second setback 0.06, or 0.0546918160678 for second setback 0.05. These match the previously documented native first-setback fold within 1e-10. All source shapes were byte-verified against this capture archive. Raw discrepancies remain; finite contact samples are not a global error proof. See cone-contact-analysis.json and the 2026-10-16-cone-contact-oracle proof.
