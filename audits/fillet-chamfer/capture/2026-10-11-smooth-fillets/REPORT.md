# Captured replay after revolution and fillet-smoothness repairs

The full run at clean kernel HEAD `012ace79` passes all 156 native entries.
All 161 captures retain their prior construction status: 70 constructed outputs,
32 operation failures, 25 import rejections, 23 inconsistent inputs, eight
unhandled intents, two timeouts and one incomplete capture. No panic occurs.

All 70 constructed outputs are geometrically consistent and preserve their input;
69 have closed Truck meshes. All 70 now prepare STEP, 69 yield native-valid
single-solid STEP, and 68 satisfy the combined limited checks, including
bidirectional sampled boundary distance <=0.001. The previous snapshot had
69, 68 and 67 respectively. The sole change in these limited pass/fail outcomes
is D4, whose original failure is now repaired.

D4 exports one valid solid with maximum sampled boundary distances 5.78032e-5
and 3.12150e-5. Native volume is 3435781.0157892; Truck STEP volume is
3435780.9991062. It also retains a closed Truck mesh, consistent geometry and
unchanged input. This supplements the independent radius, normal, knot-continuity,
volume, rigid-transform, reversed-selection and STEP-round-trip regressions in
`truck-benchmarks/results/2026-10-11-procedural-export-knots/REPORT.md`.

Q4 capture 2 still has an open Truck mesh and exports two solids, with a sampled
discrepancy of 1.20849. Its new circular fillet reaches an existing rounded roof
edge, requiring additional support/termination handling. The tolerance-grid A1
shape discrepancy, G1 trim failure and I3/J9 timeouts also remain. No full parity
verdict is awarded. All archived artifacts are verified against their captured
SHA-256 hashes; source/runtime/binary provenance is retained in the manifest.
