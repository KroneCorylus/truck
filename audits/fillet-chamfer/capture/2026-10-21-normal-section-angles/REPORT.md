# Normal-section distance-angle chamfers: clean capture

Implementation 354f307151e7a6f35c15dd2f4d072da55fb6dd81.

Added a reference-face distance-angle chamfer API. It solves the reference chord in the guide’s normal plane and the other contact at the requested angle to the new reference tangent, using the existing fitting, trimming, history and solid checks. Replay propagates typed angle constraints across the entire tangent contour. Existing distance modes keep their behavior.

451 release shapeops checks, 115 stepio checks, 65 replay tests, 13 capture tests, all-target Clippy and single-thread actions smoke pass. Native reference (8), volume (27) and distance (24) checks pass. Two geometry regressions and one replay regression fail on the committed parent. Independent controls cover 48 kernel and 48 replay operations, 15,552 contact samples, both reference faces, reverse selections, transforms, topology, immutable inputs and STEP. Twelve native controls match analytic material within 3.4e-16 and exported Truck boundaries within 6.8e-12 in sampled comparisons. One existing ignore and the known geometry precision failure remain explicit.

Clean capture: all 396 native cases pass across 648 operations. 451 constructions (+1) and 418 limited comparison passes (+0). All 450 prior successful result/comparison records, every task, native reference and selection scope are unchanged. C1’s distance-angle operation now constructs but still fails the unchanged 0.001 sampled comparison threshold. E5 now reaches construction and fails at trimming. Supporting the request does not mean its final result matches the native result.

Statuses: {"input-import-failed": 96, "operation-succeeded": 451, "operation-failed": 92, "input-inconsistent": 9}.

Status changes: [{"case": "chamfer/dist_angle/E5", "capture": "001", "changed_scope": false, "before": "unhandled-intent", "after": "operation-failed"}, {"case": "chamfer/dist_angle_sequence/C1", "capture": "001", "changed_scope": false, "before": "unhandled-intent", "after": "operation-succeeded"}].

141 of 559 positive solid operations remain unresolved; 33 are constructed discrepancies.

Archive SHA256: 7920b80fa87b9c0099e867da5f6db7542be87b82e08fd4292383da9627bb66b5.

The parent geometry tests use a documented adapter to the existing chord API with d and d*tan(angle), because the new angle API did not exist. Parent replay assertions are unchanged. General support continuation/corner collapse is not enabled for the angle solver. No parity, global-error or performance claim. No consumer/pin changes, push or PR.
