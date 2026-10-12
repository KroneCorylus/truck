# Captured replay after physical mesh refinement and exact spline joins

The full run at clean f6871a18 passes all 156 native entries. The 161
captures comprise 72 operation-succeeded, 32 operation-failed, 25
input-import-failed, 23 input-inconsistent, eight unhandled intents and one
incomplete capture. There are no timeouts or panics.

All 72 constructed outputs retain consistent geometry, unchanged input,
closed Truck meshes and native-valid single-solid STEP. Seventy meet the
combined limited comparisons including bidirectional sampled distance
<=0.001. G1 and tolerance-grid A1 remain shape discrepancies.

I3's conical-pipe fillet now completes. Its STEP volume is 979697.405982782,
versus independently integrated 979697.406914389 and native
979697.406915526. Maximum sampled distances are 0.0000113904573 and
0.0000084299785 at 1115 and 1106 stations. All 21 STEP read regressions in
the initial aa6cfa65 run recover after exact duplicate join removal. That
failed capture remains archived; Truck preparation/round trips alone had
not detected its native interoperability failures.

J9 still reaches the blend solver and fails at trim_faces. Its first side
contact cannot find an intersection with the longitudinal seam; the contact
endpoint is about 0.435 model units from that seam. Import success and a
closed unfilleted pipe do not establish blend parity.

Regressions, parent failures, independent volume derivation, release test
results, Clippy, action smoke and before/after timing medians are retained in
`truck-benchmarks/results/2026-10-12-exact-spline-joins/REPORT.md` and its
conical-mesh predecessor. The broader missing/import/construction gaps and
both shape discrepancies remain open. No full feature-parity verdict is
assigned.

Every archived case artifact was checked against its SHA-256. The manifest
retains clean source, replay binary, native runtime/library and harness
provenance; the catalog snapshot predates this evidence update.
