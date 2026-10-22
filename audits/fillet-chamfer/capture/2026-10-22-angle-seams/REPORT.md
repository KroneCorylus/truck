# Distance-angle contact seams: clean capture

Implementation 9fa6639314eab4a8f8e8de3758f27ff77e09f414.

Distance-angle chamfers now continue across tangent cylindrical and planar walls under a shared tilted planar cap. Small contact endpoint gaps are reconciled on that cap within the requested tolerance. Local refinement handles imported endpoint constraints while retaining the same fit threshold and 257-sample limit. Larger gaps still fail.

452 release shapeops tests, 115 stepio tests, 66 replay tests, 13 capture tests, all-target Clippy and single-thread actions smoke pass. Native reference (8), volume (27) and distance (24) checks pass. Two new regressions fail on the committed parent with identical test sources. Six kernel operations cover reverse selections, transformed scales, 594 contact samples, immutable inputs, topology, material and STEP. Imported replay covers both selection orientations. One existing ignore and the known geometry precision failure remain explicit.

All 396 native cases pass across 648 operations. 452 constructions (+1) and 419 limited comparison passes (+1). All 451 prior successful result/comparison records, every input task, native reference and selection scope are unchanged. The E5 distance-angle case now constructs and passes the unchanged 0.001 sampled comparison threshold.

Statuses: {"input-import-failed": 96, "operation-succeeded": 452, "operation-failed": 91, "input-inconsistent": 9}.

Status changes: [{"case": "chamfer/dist_angle/E5", "capture": "001", "changed_scope": false, "before": "operation-failed", "after": "operation-succeeded"}].

140 of 559 positive solid operations remain unresolved; 33 are constructed discrepancies.

Archive SHA256: 991dd166578b011f7688981d5fe09dac582b7521b34bb34703170545d12086e6.

No assertions or thresholds are weakened. No parity, global-error or performance claim. No consumer/pin changes, push or PR.
