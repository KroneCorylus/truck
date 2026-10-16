# Longitudinal plane-reference angles: clean full capture

Runtime 05901a28514e300c55f68cabf82fb76eeab27234: all 396 native entries pass, 648 operation records.
All requested parameters, complete scopes and native result properties unchanged.
All 352 previous constructions retain their successful checks; 352 complete
prior result/comparison records are exactly unchanged and 0 numerical
changes are recorded. No prior successful check regressed.

356 constructions (+4); 337 combined limited sampled
passes at 0.001 (+4). All constructed outputs preserve input and pass
consistency, closed mesh, STEP preparation and native-valid single-solid checks.
19 constructed discrepancies remain.
Statuses: {"input-import-failed": 96, "operation-succeeded": 356, "operation-failed": 173, "unhandled-intent": 14, "input-inconsistent": 9}.
Status changes:
- chamfer/dist_angle_complex/A7/001: unhandled-intent → operation-failed
- chamfer/dist_angle_complex/A9/001: unhandled-intent → operation-failed
- chamfer/dist_angle_sequence/A7/005: unhandled-intent → operation-succeeded
- chamfer/dist_angle_sequence/A7/006: unhandled-intent → operation-succeeded
- chamfer/dist_angle_sequence/A9/005: unhandled-intent → operation-succeeded
- chamfer/dist_angle_sequence/A9/006: unhandled-intent → operation-succeeded

Plane-reference distance-angle replay now intersects the requested planar ray
with the cylinder and uses the existing chord-to-tangent conversion. Local
branch checks remain. Cylinder-reference angles remain unsupported.
Three parent-failing tests; 20 constructed variants plus rejection guards.
Ten native controls: all Truck results preserve requested angle, contacts and
independent material. Eight native controls agree; two valid native results give
47.2948983° and20.5114600° for30° requests. Raw discrepancies remain recorded.
46 replay tests,13 capture tests,Clippy,actions smoke and final replay build pass.
Core source/integration tests unchanged from ac5b8fa3 full396-pass validation.
Proof: truck-benchmarks/results/2026-10-16-longitudinal-plane-angles/REPORT.md.
Archive: 8119 byte-verified members, SHA256 f70960b92cec3e1b9b072de19bc9678757d3ffadd11d512aa0f7d98c08e3dbfe.
Sampling does not establish full parity or a global boundary-error bound.
No consumer/pin changes, push, PR or performance claim.
