# Separated fillet components: clean capture

Implementation 569d5e3d3219bd873538be92e2e99398234d41f5.

One selection can now combine a planar corner chain with a separate curved tangent rim. Each connected component uses its existing supported builder. Input-shell order, selected-edge membership, generated-face interference and atomic failure checks remain in place.

463 release shapeops tests, 119 STEP tests, 72 replay tests, 13 capture tests, all-target Clippy and the single-thread benchmark smoke check pass. Native reference (8), volume (27) and distance (24) checks pass. Two regressions fail on the parent with identical sources and fixtures. Controls cover 24 procedural operations, one oversized rejection and two imported selection orders. One existing ignore, a test warning and the known geometry precision failure remain explicit.

All 396 native cases pass across 648 captured operations. 470 constructions (+1) and 436 limited comparison passes (+1). All 469 previous successful result/comparison records are exactly unchanged, as are every input task, native reference and selection scope.

Q2 now passes with all three selected edges: two meeting at a planar corner and a separate curved rim. Independent quarter-cylinder overlap and toroidal-rim integrals verify procedural material; exact contacts and face history are checked. The native sampled maximum is below 0.000060, and exported STEP tolerances are below 0.0000075. These checks do not establish a global error bound or full parity.

Statuses: {"input-import-failed": 95, "operation-succeeded": 470, "operation-failed": 73, "input-inconsistent": 10}.

123 of 559 positive solid operations remain unresolved; 34 are constructed discrepancies.

Archive SHA256: 2813d58e6c1382db30d12027ee065fe11ffa2e6f4b4cf87360c9fd06daf66dd0.

No assertions, thresholds or replay fitting budgets changed. No parity, global-error or performance claim. No consumer/pin changes, push or PR.
