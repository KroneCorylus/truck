# Curved terminal chamfer boundaries: clean full capture

Runtime ac5b8fa3500314e277d914a7a22233c758416808: all 396 native entries pass, 648 operation records.
Requested parameters, complete selection scopes and native result properties are
unchanged. All 312 previous constructions retain their successful checks;
312 complete prior result/comparison records are exactly unchanged and
0 have numerical changes recorded in comparison-summary.json.
No prior successful check regressed.

352 constructions (+40); 333 combined limited sampled
passes at 0.001 (+40). All constructed outputs preserve input and pass
consistent geometry, closed mesh, STEP preparation and native-valid single-solid
checks. 19 constructed discrepancies remain under these comparisons.
Statuses: {"input-import-failed": 96, "operation-succeeded": 352, "operation-failed": 171, "unhandled-intent": 20, "input-inconsistent": 9}.
Status changes:
- chamfer/dist_angle_sequence/A6/007: operation-failed → operation-succeeded
- chamfer/dist_angle_sequence/A6/008: operation-failed → operation-succeeded
- chamfer/dist_angle_sequence/A7/007: operation-failed → operation-succeeded
- chamfer/dist_angle_sequence/A7/008: operation-failed → operation-succeeded
- chamfer/dist_angle_sequence/A8/007: operation-failed → operation-succeeded
- chamfer/dist_angle_sequence/A8/008: operation-failed → operation-succeeded
- chamfer/dist_angle_sequence/A9/007: operation-failed → operation-succeeded
- chamfer/dist_angle_sequence/A9/008: operation-failed → operation-succeeded
- chamfer/dist_dist_sequence/A6/005: operation-failed → operation-succeeded
- chamfer/dist_dist_sequence/A6/006: operation-failed → operation-succeeded
- chamfer/dist_dist_sequence/A6/007: operation-failed → operation-succeeded
- chamfer/dist_dist_sequence/A6/008: operation-failed → operation-succeeded
- chamfer/dist_dist_sequence/A7/005: operation-failed → operation-succeeded
- chamfer/dist_dist_sequence/A7/006: operation-failed → operation-succeeded
- chamfer/dist_dist_sequence/A7/007: operation-failed → operation-succeeded
- chamfer/dist_dist_sequence/A7/008: operation-failed → operation-succeeded
- chamfer/dist_dist_sequence/A8/005: operation-failed → operation-succeeded
- chamfer/dist_dist_sequence/A8/006: operation-failed → operation-succeeded
- chamfer/dist_dist_sequence/A8/007: operation-failed → operation-succeeded
- chamfer/dist_dist_sequence/A8/008: operation-failed → operation-succeeded
- chamfer/dist_dist_sequence/A9/005: operation-failed → operation-succeeded
- chamfer/dist_dist_sequence/A9/006: operation-failed → operation-succeeded
- chamfer/dist_dist_sequence/A9/007: operation-failed → operation-succeeded
- chamfer/dist_dist_sequence/A9/008: operation-failed → operation-succeeded
- chamfer/equal_dist_sequence/A6/005: operation-failed → operation-succeeded
- chamfer/equal_dist_sequence/A6/006: operation-failed → operation-succeeded
- chamfer/equal_dist_sequence/A6/007: operation-failed → operation-succeeded
- chamfer/equal_dist_sequence/A6/008: operation-failed → operation-succeeded
- chamfer/equal_dist_sequence/A7/005: operation-failed → operation-succeeded
- chamfer/equal_dist_sequence/A7/006: operation-failed → operation-succeeded
- chamfer/equal_dist_sequence/A7/007: operation-failed → operation-succeeded
- chamfer/equal_dist_sequence/A7/008: operation-failed → operation-succeeded
- chamfer/equal_dist_sequence/A8/005: operation-failed → operation-succeeded
- chamfer/equal_dist_sequence/A8/006: operation-failed → operation-succeeded
- chamfer/equal_dist_sequence/A8/007: operation-failed → operation-succeeded
- chamfer/equal_dist_sequence/A8/008: operation-failed → operation-succeeded
- chamfer/equal_dist_sequence/A9/005: operation-failed → operation-succeeded
- chamfer/equal_dist_sequence/A9/006: operation-failed → operation-succeeded
- chamfer/equal_dist_sequence/A9/007: operation-failed → operation-succeeded
- chamfer/equal_dist_sequence/A9/008: operation-failed → operation-succeeded

Planar chamfers can now cross curved edges between terminal faces using the
existing curve/surface intersection routine, within the original curve bounds.
Two parent-failing regressions cover four equal/unequal, transformed/scaled and
face-order variants. The oversized-input guard passes on parent and fixed code.
Six native controls match independent section-integral material and valid STEP;
maximum sampled boundary difference 4.049468120209827e-08.
Full shapeops: 396 passed,0 failed,1 existing ignored.
42 replay tests, Clippy, actions smoke and final replay build pass.
Proof: truck-benchmarks/results/2026-10-16-curved-chamfer-terminals/REPORT.md.
Archive: 8107 byte-verified members, SHA256 efca69963094f07b78daa739bdef31fa7698bad6c252898392c902fe70cf3ad4.
Sampling does not establish full parity or a global boundary-error bound.
No consumer/pin changes, push, PR or performance claim.
