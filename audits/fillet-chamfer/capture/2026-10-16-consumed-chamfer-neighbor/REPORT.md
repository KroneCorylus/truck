# Isolated chamfers consuming short neighbors: clean full capture

Clean runtime 4211cfe0ba40ccc847279200ad4ebd15189dfb56: all 396 native entries pass, 648 operation records.
All requested parameters, scopes and native result properties are unchanged.
All 279 previous constructions retain their successful checks; 279 complete prior
result/comparison records are exactly unchanged. 0 have numerical changes,
recorded in comparison-summary.json. No prior successful check regressed.

312 constructions (+33), 292 combined limited sampled passes
at 0.001 (+33). All constructed outputs preserve input, consistent geometry,
closed mesh, STEP preparation and native-valid single-solid checks.
Statuses: {"input-import-failed": 96, "operation-succeeded": 312, "operation-failed": 199, "unhandled-intent": 32, "input-inconsistent": 9}.
Status changes:
- chamfer/dist_angle_sequence/A1/002: operation-failed → operation-succeeded
- chamfer/dist_angle_sequence/A1/003: operation-failed → operation-succeeded
- chamfer/dist_angle_sequence/A1/004: operation-failed → operation-succeeded
- chamfer/dist_angle_sequence/A1/015: operation-failed → operation-succeeded
- chamfer/dist_angle_sequence/A1/016: operation-failed → operation-succeeded
- chamfer/dist_angle_sequence/A1/017: operation-failed → operation-succeeded
- chamfer/dist_angle_sequence/A4/006: operation-failed → operation-succeeded
- chamfer/dist_angle_sequence/A4/008: operation-failed → operation-succeeded
- chamfer/dist_angle_sequence/C2/003: operation-failed → operation-succeeded
- chamfer/dist_angle_sequence/C2/008: operation-failed → operation-succeeded
- chamfer/dist_angle_sequence/C2/009: operation-failed → operation-succeeded
- chamfer/dist_dist_sequence/A1/002: operation-failed → operation-succeeded
- chamfer/dist_dist_sequence/A1/004: operation-failed → operation-succeeded
- chamfer/dist_dist_sequence/A1/015: operation-failed → operation-succeeded
- chamfer/dist_dist_sequence/A1/016: operation-failed → operation-succeeded
- chamfer/dist_dist_sequence/A4/005: operation-failed → operation-succeeded
- chamfer/dist_dist_sequence/A4/006: operation-failed → operation-succeeded
- chamfer/dist_dist_sequence/A4/008: operation-failed → operation-succeeded
- chamfer/dist_dist_sequence/C2/003: operation-failed → operation-succeeded
- chamfer/dist_dist_sequence/C2/008: operation-failed → operation-succeeded
- chamfer/dist_dist_sequence/C2/009: operation-failed → operation-succeeded
- chamfer/equal_dist_sequence/A1/002: operation-failed → operation-succeeded
- chamfer/equal_dist_sequence/A1/004: operation-failed → operation-succeeded
- chamfer/equal_dist_sequence/A1/015: operation-failed → operation-succeeded
- chamfer/equal_dist_sequence/A1/016: operation-failed → operation-succeeded
- chamfer/equal_dist_sequence/A1/017: operation-failed → operation-succeeded
- chamfer/equal_dist_sequence/A4/005: operation-failed → operation-succeeded
- chamfer/equal_dist_sequence/A4/006: operation-failed → operation-succeeded
- chamfer/equal_dist_sequence/A4/007: operation-failed → operation-succeeded
- chamfer/equal_dist_sequence/A4/008: operation-failed → operation-succeeded
- chamfer/equal_dist_sequence/C2/003: operation-failed → operation-succeeded
- chamfer/equal_dist_sequence/C2/008: operation-failed → operation-succeeded
- chamfer/equal_dist_sequence/C2/009: operation-failed → operation-succeeded

The existing straight-edge solver is now tried for an isolated chamfer after a
miter route rejects a consumed neighboring edge. Equal and unequal entry points are
covered; if the straight solver rejects the input, the original diagnostic stays.
Two new tests fail on parent ed603e7d and pass unchanged; six variants check
analytic overlap volume, generated plane/normals, consumed edge removal, face
count, unchanged input, topology/mesh and robust STEP. The former sequential-gap expectation now requires successful material, solid,
STEP, history and convexity checks on its same four axis-pair cases; it also
fails on the parent. Existing oversized and collision guards pass unchanged. Three native controls pass analytic material
and valid single-solid STEP, maximum sampled boundary difference 5.0242958677880805e-15.
Full shapeops: 393 passed, 0 failed, 1 existing ignored;
39 replay tests, Clippy, actions smoke and final replay build pass.
Proof: truck-benchmarks/results/2026-10-16-consumed-chamfer-neighbor/REPORT.md.
Archive: 7987 byte-verified members, SHA256 37ea90f8f53e02bc1d334aa064a82f25396717db9d21447645a00b0800858cf1.
Finite sampling is not full parity or a global bound. No speed claim.
