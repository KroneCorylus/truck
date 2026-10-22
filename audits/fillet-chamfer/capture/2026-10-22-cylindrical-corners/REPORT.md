# Cylinder rim fillet corners: clean capture

Implementation 4ffc8523107c00d692bcc14218dec2df56b04b34.

Three selected edges can now meet between an outer cylinder, a perpendicular cap and a radial plane. Exact offset intersections place a spherical corner shared by two cylindrical rounds and a toroidal rim. Circular contact arcs preserve the trimmed sweep, including cuts beyond the original rim midpoint.

458 release shapeops tests, 119 STEP tests, 69 replay tests, 13 capture tests, all-target Clippy and the single-thread benchmark smoke check pass. Native reference (8), volume (27) and distance (24) checks pass. Two regressions fail on the parent with identical sources and fixtures. Controls cover 24 successful variants, four collision rejections, 3,672 tangent-normal pairs and four imported selections at two STEP fitting budgets. One existing ignore, an existing test warning and the known geometry precision failure remain explicit.

All 396 native cases pass across 648 captured operations. 462 constructions (+2) and 429 limited comparison passes (+2). All 460 previous successful result/comparison records are exactly unchanged, as are every input task, native reference and selection scope.

New limited passes: blend/simple/B3/001 and blend/simple/B7/001.

Statuses: {"input-import-failed": 95, "operation-succeeded": 462, "operation-failed": 81, "input-inconsistent": 10}.

130 of 559 positive solid operations remain unresolved; 33 are constructed discrepancies.

Independent section integration gives B3 volume 190760.57996317057 and B7 volume 580239.2966730265, within 1.2e-9 absolute volume of native.

Archive SHA256: e5214f1736ce1964cd369e0f6411f02a49449842a6ed634eb010433d87413900.

No assertions, thresholds or replay fitting budgets changed. No global-error, parity or performance claim. No consumer/pin changes, push or PR.
