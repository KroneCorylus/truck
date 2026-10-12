# Captured replay after tangent support continuation

The full run at clean kernel HEAD a83a0f5c passes all 156 native entries. Its 161
captures retain the previous construction statuses: 70 operation-succeeded,
32 operation-failed, 25 input-import-failed, 23 input-inconsistent, eight
unhandled intents, two timeouts and one incomplete capture. There are no panics.

All 70 constructed outputs are now geometrically consistent, preserve their
inputs, have closed Truck meshes, prepare STEP and yield native-valid single-solid
STEP exports. Sixty-nine satisfy the combined limited checks, including sampled
bidirectional boundary distance <=0.001. The prior snapshot had 69 closed meshes,
69 native-valid single-solid exports and 68 combined limited passes.

The sole limited pass/fail change is Q4 capture 2. Its post-root fillet now rolls
from the plane onto the tangent rounded roof rather than returning intersecting
planar loops. Native volume is 149.08377356777387, Truck STEP volume is
149.08379605764409; the independent regression integral is approximately
149.083817359215. Maximum sampled boundary distances are 0.00032652459800665539
and 0.0003299252872722417. The result has one valid solid and a closed Truck mesh.
The scale, transform, reversal, independent geometry/volume and STEP checks are
recorded in truck-benchmarks/results/2026-10-12-roof-support-continuation/REPORT.md.

The remaining constructed-result discrepancy is tolerance-grid A1. G1 still
fails during face trimming, and I3/J9 still time out. Other import, input and
construction failures remain. No full feature-parity verdict is awarded.
Every archived artifact is checked against its manifest SHA-256; clean source,
replay binary, native runtime/library and harness provenance are retained.
