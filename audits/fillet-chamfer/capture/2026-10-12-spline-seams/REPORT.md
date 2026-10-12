# Captured replay after exact spline seam import

The full run at clean kernel HEAD aece8e35 passes all 156 native entries. The
161 captures comprise 71 operation-succeeded, 32 operation-failed, 25
input-import-failed, 23 input-inconsistent, eight unhandled intents, one
timeout and one incomplete capture. There are no panics.

All 71 constructed outputs retain consistent geometry, unchanged input,
closed Truck meshes and native-valid single-solid STEP. Sixty-nine meet
the combined limited comparisons including bidirectional sampled distance
<=0.001. G1 and tolerance-grid A1 remain shape discrepancies.

J9 is the sole construction-status change. Its split spline pipe seams now
convert to exact spatial splines, and the imported input passes consistency
and selection. Its two selected rim segments reach the blend solver, which
still fails at trim_faces with unchanged input. I3 remains the only timeout,
in mesh refinement. No successful J9 fillet or full case parity is claimed.

Independent pipe volume, closed mesh and STEP checks, section/derivative
regressions, parent failures, 425 passing release STEP/shapeops tests, Clippy
and action smoke are recorded in
truck-benchmarks/results/2026-10-12-spline-seams/REPORT.md.
Every archived case artifact was checked against its SHA-256. Clean source,
replay binary, native runtime/library and harness provenance are retained;
the catalog snapshot predates this evidence update.
