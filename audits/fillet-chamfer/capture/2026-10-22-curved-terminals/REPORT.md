# Curved terminal fillet corners: clean capture

Implementation da62f251d7f620451f415f992e1a263731637fd1.

Planar multi-edge fillet corners now terminate on curved faces. The construction retains exact cylindrical, spherical and toroidal blend geometry, intersects terminal supports, and trims neighboring conic edges with shared topology.

454 release shapeops tests, 119 STEP tests, 67 replay tests, 13 capture tests, all-target Clippy and the single-thread benchmark smoke check pass. Native reference (8), volume (27) and distance (24) checks pass. Two regressions fail on the parent with identical sources and fixtures. Controls cover 32 successful variants, eight collision rejections, 5,168 tangent-normal pairs and eight imported selections at two STEP fitting budgets. One existing ignore and the known geometry precision failure remain explicit.

All 396 native cases pass across 648 captured operations. 459 constructions (+7) and 426 limited comparison passes (+7). All 452 prior successful result/comparison records are exactly unchanged, as are every input task, native reference and selection scope.

New limited passes: blend/simple/B1, blend/simple/B5, blend/simple/B9, blend/simple/C4, blend/simple/D3, blend/simple/D7, blend/simple/F7.

Statuses: {"input-import-failed": 95, "operation-succeeded": 459, "operation-failed": 84, "input-inconsistent": 10}.

133 of 559 positive solid operations remain unresolved; 33 are constructed discrepancies.

Additional independent section integrals for spherical D3/D7 agree with native material within 5e-7 absolute volume. F7 is established by the unchanged corpus checks; no independent F7 material integral is claimed. G1/H4 native-reference diagnoses are archived separately without changing comparison statuses.

Archive SHA256: abad2950f7c0a005f9c98a36731062ef9da7377c9f930cd66fd989af233cf6b4.

No assertions, thresholds or replay fitting budgets changed. No global-error, parity or performance claim. No consumer/pin changes, push or PR.
