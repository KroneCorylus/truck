# Spherical support corners: clean capture

Implementation 98af2566b52af1fba72b8726bddf0a5219ddc7f5.

Three selected edges on an outward sphere and two planes now join an exact rolling sphere, two toroidal blends and a cylindrical round. Contact curves trim against their remote planar boundaries. Existing geometry, closure and trim checks remain; consumed terminal faces reject without changing input.

465 release shapeops tests, 119 STEP tests, 73 replay tests, 13 capture tests, all-target Clippy and the single-thread benchmark smoke check pass. Native reference (8), volume (27) and distance (24) checks pass. Two regressions fail on the parent with identical sources and fixtures. Controls cover 36 procedural operations, nine oversized rejections and six normalized imported selections. Three original-scale production replays pass their construction checks. One existing ignore, a test warning and the known geometry precision failure remain explicit.

All 396 native cases pass across 648 captured operations. 473 constructions (+3) and 437 limited comparison passes (+1). All 470 previous successful result/comparison records are exactly unchanged, as are every input task, native reference and selection scope.

D5 now passes the unchanged sampled-distance threshold. D9 and E4 now construct but remain comparison discrepancies. Their native corner geometry differs from exact rolling spheres. E4 also exposes inconsistent native surface-volume integration: tessellation of the exported STEP agrees with the raw solid and independent material integral within the chord budget. The canonical comparator and its statuses remain unchanged. These checks do not establish a global error bound or full parity.

Statuses: {"input-import-failed": 95, "operation-succeeded": 473, "operation-failed": 70, "input-inconsistent": 10}.

122 of 559 positive solid operations remain unresolved; 36 are constructed discrepancies.

Archive SHA256: a26fbd4333cb0755162af8ada957005a9295bf43bdc03abede116493badb12a2.

No assertions, thresholds or replay fitting budgets changed. No parity, global-error or performance claim. No consumer/pin changes, push or PR.
