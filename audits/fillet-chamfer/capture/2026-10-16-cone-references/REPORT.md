# Plane/cone chamfer references: clean full capture

Clean runtime 54b272d6cc3f82044adeb9172e38891103dbf9cb: all 396 native entries pass, 648 operation records.
Requested parameters, contour scopes and native result properties are unchanged.
42 records gained optional cone identity metadata. All 241 previously successful
replay results and complete comparisons remain exactly unchanged; no prior
successful check regressed.

279 constructions (+38), 259 combined limited sampled comparisons at 0.001 (+29).
All 279 preserve input and pass consistency, mesh closure, STEP preparation and
native valid-single-solid checks. Nine newly constructed unequal-distance records
retain sampled discrepancies: dist_dist/C1,C6,C7,D1,D2,D5,D6,D9,E1. Their maximum
sampled difference ranges from 0.02174 to 0.04349. They are not counted as passes.
The previous eleven discrepancies also remain, for twenty total. Another eighteen
records now reach construction but fail; these remain unresolved.
Statuses: {"input-import-failed": 96, "operation-succeeded": 279, "operation-failed": 232, "unhandled-intent": 32, "input-inconsistent": 9}.
Complete native scopes: 487 chamfer and 161 fillet records.

Status changes:
- chamfer/dist_angle/B3/001: unhandled-intent → operation-failed
- chamfer/dist_angle/B4/001: unhandled-intent → operation-failed
- chamfer/dist_angle/B5/001: unhandled-intent → operation-succeeded
- chamfer/dist_angle/B6/001: unhandled-intent → operation-succeeded
- chamfer/dist_angle/B7/001: unhandled-intent → operation-failed
- chamfer/dist_angle/B8/001: unhandled-intent → operation-failed
- chamfer/dist_angle/B9/001: unhandled-intent → operation-failed
- chamfer/dist_angle/C1/001: unhandled-intent → operation-succeeded
- chamfer/dist_angle/C2/001: unhandled-intent → operation-failed
- chamfer/dist_angle/C3/001: unhandled-intent → operation-failed
- chamfer/dist_angle/C4/001: unhandled-intent → operation-failed
- chamfer/dist_angle/C5/001: unhandled-intent → operation-failed
- chamfer/dist_angle/C6/001: unhandled-intent → operation-succeeded
- chamfer/dist_angle/C7/001: unhandled-intent → operation-succeeded
- chamfer/dist_angle/C8/001: unhandled-intent → operation-succeeded
- chamfer/dist_angle/C9/001: unhandled-intent → operation-succeeded
- chamfer/dist_angle/D1/001: unhandled-intent → operation-succeeded
- chamfer/dist_angle/D2/001: unhandled-intent → operation-succeeded
- chamfer/dist_angle/D3/001: unhandled-intent → operation-succeeded
- chamfer/dist_angle/D4/001: unhandled-intent → operation-succeeded
- chamfer/dist_angle/D5/001: unhandled-intent → operation-succeeded
- chamfer/dist_angle/D6/001: unhandled-intent → operation-succeeded
- chamfer/dist_angle/D7/001: unhandled-intent → operation-succeeded
- chamfer/dist_angle/D8/001: unhandled-intent → operation-succeeded
- chamfer/dist_angle/D9/001: unhandled-intent → operation-succeeded
- chamfer/dist_angle/E1/001: unhandled-intent → operation-succeeded
- chamfer/dist_angle/E2/001: unhandled-intent → operation-succeeded
- chamfer/dist_angle/E3/001: unhandled-intent → operation-succeeded
- chamfer/dist_dist/B3/001: unhandled-intent → operation-failed
- chamfer/dist_dist/B4/001: unhandled-intent → operation-failed
- chamfer/dist_dist/B5/001: unhandled-intent → operation-succeeded
- chamfer/dist_dist/B6/001: unhandled-intent → operation-succeeded
- chamfer/dist_dist/B7/001: unhandled-intent → operation-failed
- chamfer/dist_dist/B8/001: unhandled-intent → operation-failed
- chamfer/dist_dist/B9/001: unhandled-intent → operation-failed
- chamfer/dist_dist/C1/001: unhandled-intent → operation-succeeded
- chamfer/dist_dist/C2/001: unhandled-intent → operation-failed
- chamfer/dist_dist/C3/001: unhandled-intent → operation-failed
- chamfer/dist_dist/C4/001: unhandled-intent → operation-failed
- chamfer/dist_dist/C5/001: unhandled-intent → operation-failed
- chamfer/dist_dist/C6/001: unhandled-intent → operation-succeeded
- chamfer/dist_dist/C7/001: unhandled-intent → operation-succeeded
- chamfer/dist_dist/C8/001: unhandled-intent → operation-succeeded
- chamfer/dist_dist/C9/001: unhandled-intent → operation-succeeded
- chamfer/dist_dist/D1/001: unhandled-intent → operation-succeeded
- chamfer/dist_dist/D2/001: unhandled-intent → operation-succeeded
- chamfer/dist_dist/D3/001: unhandled-intent → operation-succeeded
- chamfer/dist_dist/D4/001: unhandled-intent → operation-succeeded
- chamfer/dist_dist/D5/001: unhandled-intent → operation-succeeded
- chamfer/dist_dist/D6/001: unhandled-intent → operation-succeeded
- chamfer/dist_dist/D7/001: unhandled-intent → operation-succeeded
- chamfer/dist_dist/D8/001: unhandled-intent → operation-succeeded
- chamfer/dist_dist/D9/001: unhandled-intent → operation-succeeded
- chamfer/dist_dist/E1/001: unhandled-intent → operation-succeeded
- chamfer/dist_dist/E2/001: unhandled-intent → operation-succeeded
- chamfer/dist_dist/E3/001: unhandled-intent → operation-succeeded

Capture/replay now preserves cone apex, opening direction and half-angle, matches
one adjacent seed face, and converts angles on circular plane/cone rims with the
correct oriented dihedral. Geometry solvers and fit tolerances are unchanged.
Three parent-failing regressions cover sixteen constructed variants. All39 replay
and13 capture tests, Clippy, action smoke and final replay build pass. Eight fresh
native controls pass analytic material and valid-single-STEP comparisons, with
maximum sampled distance1.4734521025516756e-7.
Proof: truck-benchmarks/results/2026-10-16-cone-references/REPORT.md.
Archive: 7888 byte-verified members, SHA256 060b5cb08f7f4f051009746697a692a7f11bc20926a6e4e731c577181c83a15a.
Finite sampling is not complete parity or a global-distance certificate.
