# Captured replay after two-edge cap trimming

The full run at clean kernel HEAD 83eccc89 passes all 156 native entries. The
161 captures comprise 71 operation-succeeded, 31 operation-failed, 25
input-import-failed, 23 input-inconsistent, eight unhandled intents, two
timeouts and one incomplete capture. There are no panics.

All 71 constructed outputs have consistent geometry, unchanged inputs, closed
Truck meshes, prepared STEP and native-valid single-solid STEP exports.
Sixty-nine satisfy the combined limited comparisons, including bidirectional
sampled boundary distance <=0.001. The previous clean capture constructed 70,
with the same 69 limited passes.

G1 is the sole construction-status change. Its parabolic rim previously failed
at trim_faces because the supporting cap has only two boundary edges. Both
ends can now trim the common cap edge and terminal face. Truck STEP volume is
132185649.97246; independent rolling-envelope integration converges to
132185649.98388772. The direct regression also checks radius, normals, topology,
closed mesh, STEP, rigid transforms, reversed selection and input immutability.
Its derivation is in truck-shapeops/tests/TWO_EDGE_BLEND_CAPS.md; parent failures
and validation are in truck-benchmarks/results/2026-10-12-two-edge-caps/REPORT.md.

G1 does not pass the native sampled comparison: OCCT volume is
132189932.40065464, with maximum distances 0.98749869748551278 native-to-Truck
and 0.018545749048056535 Truck-to-native. The independent checks support the
new construction, but this discrepancy remains explicit. Tolerance-grid A1
is the other constructed-result discrepancy; I3 and J9 still time out.
Import, input and other construction failures remain. No full feature-parity
verdict is awarded.

Every archived case artifact was checked against its manifest SHA-256. The
manifest retains clean source, replay binary, native library/runtime and
harness provenance; its catalog snapshot predates this evidence update.
