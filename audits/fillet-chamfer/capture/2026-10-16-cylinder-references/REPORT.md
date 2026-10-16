# Cylindrical chamfer references: clean full capture

Clean runtime f96cb1051acc83eed1e142b1d8ae062f874ead54: all 396 native entries pass, 648 operation records.
Requests, contour scopes and native reference properties are unchanged. Optional
cylinder identity metadata was added to 40 records. All 231 previously
successful replay results and complete comparisons are exactly unchanged.
No prior successful check regressed. Twelve records now reach construction but
still fail there; these remain unresolved, not regressions of successful cases.

Statuses: {"input-import-failed": 96, "operation-succeeded": 241, "operation-failed": 214, "unhandled-intent": 88, "input-inconsistent": 9}.
Constructed: 241, an increase of 10; combined limited sampled
comparisons at 0.001: 230, an increase of 10.
All constructed outputs preserve input and pass consistency, closed mesh, STEP
preparation and native valid-single-solid checks (see comparison-summary.json).
Complete native scopes: 487 chamfer and 161 fillet records.

Status changes:
- chamfer/dist_angle/A5/001: unhandled-intent → operation-failed
- chamfer/dist_angle/A6/001: unhandled-intent → operation-failed
- chamfer/dist_angle/A8/001: unhandled-intent → operation-failed
- chamfer/dist_angle/B1/001: unhandled-intent → operation-failed
- chamfer/dist_angle_complex/C3/001: unhandled-intent → operation-failed
- chamfer/dist_angle_sequence/A6/001: unhandled-intent → operation-succeeded
- chamfer/dist_angle_sequence/A6/004: unhandled-intent → operation-succeeded
- chamfer/dist_angle_sequence/A8/001: unhandled-intent → operation-succeeded
- chamfer/dist_angle_sequence/A8/004: unhandled-intent → operation-succeeded
- chamfer/dist_angle_sequence/C3/002: unhandled-intent → operation-failed
- chamfer/dist_angle_sequence/C3/003: unhandled-intent → operation-succeeded
- chamfer/dist_dist/A5/001: unhandled-intent → operation-failed
- chamfer/dist_dist/A6/001: unhandled-intent → operation-failed
- chamfer/dist_dist/A8/001: unhandled-intent → operation-failed
- chamfer/dist_dist/B1/001: unhandled-intent → operation-failed
- chamfer/dist_dist_complex/C3/001: unhandled-intent → operation-failed
- chamfer/dist_dist_sequence/A6/001: unhandled-intent → operation-succeeded
- chamfer/dist_dist_sequence/A6/004: unhandled-intent → operation-succeeded
- chamfer/dist_dist_sequence/A8/001: unhandled-intent → operation-succeeded
- chamfer/dist_dist_sequence/A8/004: unhandled-intent → operation-succeeded
- chamfer/dist_dist_sequence/C3/002: unhandled-intent → operation-failed
- chamfer/dist_dist_sequence/C3/003: unhandled-intent → operation-succeeded

The change preserves cylindrical face identity and side assignments in chamfer
capture/replay. It does not modify the geometry solver or fit tolerances.
Three parent-failing replay tests pass unchanged; 36 replay tests, 13 capture
tests, Clippy, action smoke and final replay build pass. Three fresh native
controls agree with analytic material and export valid single-solid STEP results,
with maximum sampled boundary difference below 7e-9.
Implementation proof: truck-benchmarks/results/2026-10-16-cylinder-references/REPORT.md.
Archive: 7774 byte-verified members, SHA256 1f277318bf25a6b50aae270755a6955981eceb5e9c01ea6755f3b34dfda956a0.
Sampling is not a global-distance or full-parity certificate. No performance claim.
