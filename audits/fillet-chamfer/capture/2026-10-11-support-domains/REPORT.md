# Full captured replay after imported-support and selection fixes

The native instrumented run still passes all 156 entries. At clean kernel HEAD
`4c764bef`, 148 entries invoke blend and produce 161 captures. Truck records:

- 64 constructed outputs, versus 45 in the original baseline;
- 31 import rejections, 23 inconsistent inputs and 31 operation failures;
- 8 unhandled intents, 2 timeouts, 1 preprocessing panic and 1 incomplete capture.

All 64 constructed results are geometrically consistent, preserve input and have
closed Truck meshes. Sixty-three export STEP accepted by native OCCT as a valid
single solid. Sixty-two also meet the bidirectional sampled boundary limit of
0.001. None receives a full parity verdict.

A9 and the previously panicking F8 now construct successfully. A9's maximum
sampled discrepancy is below 1.81e-13; F8 is below 8.41e-6. I1's previously open
Truck mesh is now closed. D4 still fails STEP preparation and the tolerance-grid
A1 shape discrepancy remains about 0.02238. I3/J9 still time out, and Z1 still
panics during preprocessing in this snapshot.

The corrected physical matcher excludes a straight chord sharing a curved
selection's endpoints. G1 now selects its intended single edge and reaches face
trimming, where construction fails. F9/G2 remain unresolved multi-edge tasks.
Kernel and matcher regressions are documented in:

- `truck-benchmarks/results/2026-10-11-imported-support-domains/REPORT.md`
- `truck-benchmarks/results/2026-10-11-physical-selection-chords/REPORT.md`

The earlier baseline and intermediate captures remain separately available.
Every listed artifact, the archive, actual kernel sources, replay/native
executables, native harness and fixtures are hashed. The runtime sources match
commits `4a3e11d4` and `4c764bef`. Finite checks do not certify global intersections,
all contact constraints, persistent editing or practical latency.
