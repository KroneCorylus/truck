# Captured replay after active spline-domain import

The full run at clean kernel HEAD eaa603bb passes all 156 native entries. The
161 captures comprise 71 operation-succeeded, 31 operation-failed, 26
input-import-failed, 23 input-inconsistent, eight unhandled intents, one
timeout and one incomplete capture. There are no panics.

All 71 constructed results retain consistent geometry, unchanged inputs,
closed Truck meshes and native-valid single-solid STEP. Sixty-nine satisfy
the combined limited comparisons, including bidirectional sampled distance
<=0.001. G1 and tolerance-grid A1 remain shape discrepancies.

J9 is the only construction-status change: its timeout becomes an explicit
`Unsupported imported geometry` rejection. Imported padded NURBS knot vectors
are now restricted to their active curve/surface domains, preventing NaN
edge sampling during preprocessing. A seam pcurve generated when splitting
the closed spline pipe is still unsupported by the modeling conversion.
This does not establish successful filleting of J9. I3 remains the sole
timeout, in mesh refinement after construction.

The independent polynomial/rational geometry regressions, parent failures,
422 passing release STEP/shapeops tests, clean Clippy and action smoke check
are recorded in truck-benchmarks/results/2026-10-12-unclamped-domains/REPORT.md.
All archived case artifacts were verified against their SHA-256 hashes;
clean source, binary, native runtime/library and harness provenance are
retained. The catalog snapshot predates this evidence update. No full
feature-parity verdict is assigned.
