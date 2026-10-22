# Extruded spline parameter curves: clean capture

Implementation fc7757a214dd571c3c11dd99fe4c33e01692bdec.

B-spline parameter curves on exactly parameterized extruded supports now import without approximation. Finite extrusion domains include complete parameter-curve bounds, including small endpoint extrapolation from STEP round trips. Malformed and collapsed traces remain rejected.

452 release shapeops tests, 119 stepio tests, 66 replay tests, 13 capture tests, all-target Clippy and single-thread actions smoke pass. Native reference (8), volume (27) and distance (24) checks pass. Three regressions fail on the committed parent with identical tests and fixtures. Controls cover 64 exact traces, 8,256 point/tangent/curvature samples and 16 imported face domains with 8,256 boundary samples. The actual C1 fixture also passes exact trace and face-domain checks. One existing ignore and the known geometry precision failure remain explicit.

All 396 native cases pass across 648 operations. 452 constructions (+0) and 419 limited comparison passes (+0). All 452 prior successful result/comparison records, every input task, native reference and selection scope are unchanged. C1 equal-distance capture 006 moves from input-import-failed to input-inconsistent: its missing representation is supported, while its remaining off-surface edges still fail strict consistency. This checkpoint does not add a successful chamfer or comparison pass.

Statuses: {"input-import-failed": 95, "operation-succeeded": 452, "operation-failed": 91, "input-inconsistent": 10}.

Status changes: [{"case": "chamfer/equal_dist_sequence/C1", "capture": "006", "changed_scope": false, "before": "input-import-failed", "after": "input-inconsistent"}].

140 of 559 positive solid operations remain unresolved; 33 are constructed discrepancies.

Archive SHA256: 10a7d3c28611c128ea1fc83c3437e81487f1a48cc45fd0651a95873cbd79ab63.

No assertions or thresholds are weakened. No parity, global-error or performance claim. No consumer/pin changes, push or PR.
