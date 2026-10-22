# Open chamfer terminals: clean capture

Implementation cb0ae1b78d664ad38b1974029b2fe721e9b9ccef.

Open cylinder/plane chamfer contacts now reach curved terminal boundaries. Changed cylindrical terminal faces can expand their parameter domains while retaining the same analytic surface. Internal seam stations remain shared; already consistent faces stay unchanged.

461 release shapeops tests, 119 STEP tests, 71 replay tests, 13 capture tests, all-target Clippy and the single-thread benchmark smoke check pass. Native reference (8), volume (27) and distance (24) checks pass. Two regressions fail on the parent with identical sources and fixtures. Controls cover 24 procedural operations, two oversized rejections and six imported selections. One existing ignore, a test warning and the known geometry precision failure remain explicit.

All 396 native cases pass across 648 captured operations. 469 constructions (+6) and 435 limited comparison passes (+6). All 463 prior successes retain every validation result and sampled distance. 460 result/comparison records are exactly unchanged. Three E5 records have numerical volume changes: two at roundoff level, and the angle case by 2.38e-7 in STEP volume. Direct before/after sampled distance is below 6.7e-6; repeated outputs agree below 3.6e-10. Input tasks, native references and selection scopes are unchanged.

C3 combined capture 001 (26 selected edges) and sequence capture 002 (one selected edge) now pass equal-distance, two-distance and reference-angle modes. Independent disk-intersection section integrals verify procedural material. Native sampled distances remain below 0.000063, and imported STEP edge/vertex tolerances remain below 0.00028. These checks do not establish a global error bound or full parity.

E5 numerical differences arise in continued open contact trimming/refitting. All existing tilted-cap regression checks pass unchanged. Before/after and repeat comparisons are archived under E5-check; they are sampled evidence, not an exact-geometry proof.

Statuses: {"input-import-failed": 95, "operation-succeeded": 469, "operation-failed": 74, "input-inconsistent": 10}.

124 of 559 positive solid operations remain unresolved; 34 are constructed discrepancies.

Archive SHA256: 37b93bf4e0f54debe8fef31249ebedf25b41e62fe0470d9b090229857bb1ffa5.

No assertions, thresholds or replay fitting budgets changed. No parity, global-error or performance claim. No consumer/pin changes, push or PR.
