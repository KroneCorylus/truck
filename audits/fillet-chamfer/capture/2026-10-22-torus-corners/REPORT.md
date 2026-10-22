# Toroidal support corners: clean capture

Implementation 066b4880e2d0f3d5f573cd4547312ad963796f45.

Three convex selected edges on an outward torus and two planes now join an exact rolling sphere, toroidal cap rounds and fitted torus/radial-plane blends. Offset torus intersections and minor-angle rolling centers preserve the meridian branch, including nonconvex profiles. Remote contacts intersect their terminal before shared trim assembly; existing guards remain.

469 release shapeops tests, 119 STEP tests, 75 replay tests, 13 capture tests, all-target Clippy and the single-thread benchmark smoke check pass. Native reference (8), volume (27) and distance (24) checks pass. Two regressions fail on the parent with identical sources and fixtures. Controls cover 36 procedural operations, six oversized rejections, one mixed-sign rejection and six normalized imported selections. Three original-scale production replays pass their construction checks. One existing ignore, two existing test warnings and the known geometry precision failure remain explicit.

All 396 native cases pass across 648 captured operations. 480 constructions (+3) and 437 limited comparison passes (+0). All 477 previous successful result/comparison records are exactly unchanged, as are every input task, native reference and selection scope.

This complete capture used one worker. The previous cone checkpoint retains the kernel-confirmed out-of-memory attempt and the exact serial retry that established this capture policy.

E6, E8 and F3 now construct but remain sampled-comparison discrepancies. Independent material integration checks both raw and STEP solids; the nonconvex F3 profile subtracts its rounded inner cavity from the outer dome. The 128/256-node controls agree within 1e-7, and radius-0.1 refinement to 512 nodes changes the result by less than 5e-9. Native comparisons and thresholds remain unchanged; no global-error bound or full parity is claimed. F1 mixed convex/concave corners remain unsupported in this committed construction.

Statuses: {"input-import-failed": 95, "operation-succeeded": 480, "operation-failed": 63, "input-inconsistent": 10}.

122 of 559 positive solid operations remain unresolved; 43 are constructed discrepancies.

Archive SHA256: 0bad14be0c383aa48050e7941d3311ca8665946be5e6db9f145d5e02e52ba9b8.

No assertions, thresholds or replay fitting budgets changed. No parity, global-error or performance claim. No consumer/pin changes, push or PR.
