# Chamfer transitions: clean capture

Implementation ee89b04238c37491cf424ca529c983ac08c652d0.

Unequal convex chamfer transitions on nonconvex solids now orient against the surviving adjacent boundary. This repairs two reversed triangles in the native A1 case while retaining the existing miter geometry and validity checks.

459 release shapeops tests, 119 STEP tests, 70 replay tests, 13 capture tests, all-target Clippy and the single-thread benchmark smoke check pass. Native reference (8), volume (27) and distance (24) checks pass. Two regressions fail on the parent with identical sources and fixtures. Controls cover 64 procedural operations with independently integrated material and two imported selection orders. Existing mixed chamfer tests pass unchanged. One existing ignore, a test warning and the known geometry precision failure remain explicit.

All 396 native cases pass across 648 captured operations. 463 constructions (+1) and 429 limited comparison passes (+0). All 462 previous successful result/comparison records are exactly unchanged, as are every input task, native reference and selection scope.

A1 is a construction improvement, not a resolved comparison. Its valid closed STEP solid has volume 1.0931899122619453 versus native 1.0929679122618834. Sampled distances remain approximately 0.0212 and 0.0289. Corner geometry still differs.

Statuses: {"input-import-failed": 95, "operation-succeeded": 463, "operation-failed": 80, "input-inconsistent": 10}.

130 of 559 positive solid operations remain unresolved; 34 are constructed discrepancies.

Archive SHA256: d13e939cfe8387020ec56be3bcff37baf9524ba5780186d6e26b32e0bcb92c9c.

No assertions, thresholds or replay fitting budgets changed. No parity, global-error or performance claim. No consumer/pin changes, push or PR.
