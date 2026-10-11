# Captured replay after tangent-input preprocessing

The full instrumented native run passes all 156 entries at clean kernel HEAD
`6351c68b`, containing production repair `9285a3a0`. The 161 captures now include
70 constructed outputs, versus 64 before the repair and 45 in the first baseline.
The former preprocessing panic is gone. Remaining statuses are 25 import
rejections, 23 inconsistent inputs, 32 operation failures, eight unhandled intents,
two timeouts and one incomplete capture. No full parity verdict is awarded.

All 70 constructed outputs are geometrically consistent and preserve their input;
69 have closed Truck meshes and 69 prepare STEP. Native OCCT accepts 68 STEP
outputs as valid single solids. Sixty-seven also satisfy the bidirectional sampled
boundary threshold of 0.001 and the other limited checks.

Z1 now constructs successfully from its tangent stacked-cylinder input, with a
closed mesh and valid single-solid STEP. Maximum sampled boundary distances are
3.48020e-5 and 3.47948e-5. Native volume is 7840.79898846; Truck STEP volume is
7840.79812169. This is a measured successful physical task, not global proof of
surface accuracy, contact continuity, collision freedom or edit stability.

Newly reachable tasks also expose additional problems. Q4's second capture
constructs geometry with an open Truck mesh and exports two solids instead of one;
its maximum sampled discrepancy is 1.20840. D4 still fails STEP preparation.
Tolerance-grid A1 remains outside the sample threshold, and I3/J9 still time out.
These failures remain visible; import progress is not counted as parity.

All archived files are checked against the captured SHA-256 hashes. Source and
binary hashes record the clean snapshot before subsequent mesh/export work began.
The preprocessing regression and validation scope are documented in
`truck-benchmarks/results/2026-10-11-tangent-face-splitting/REPORT.md`.
