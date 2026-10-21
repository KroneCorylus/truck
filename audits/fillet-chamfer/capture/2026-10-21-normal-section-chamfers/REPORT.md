# Normal-section chamfers: clean capture

Implementation 40aa15af90b7969e5ead45de7f4dc0b30d804bd8.

Added an explicit chamfer solver for straight-line setbacks in the edge’s normal section, sharing the existing fitting, trimming and solid checks. Captured point-and-normal references now identify non-elementary faces uniquely, allowing unequal-distance replay on those supports. Existing tangent-projection APIs and consumer calls retain their behavior.

448 release shapeops checks, 115 stepio checks, 64 replay tests, 13 capture tests, all-target Clippy and single-thread actions smoke pass. Eight native reference checks and 27 native volume checks pass. Two geometry tests, two replay tests and four native reference checks fail on the committed parent. Independent controls cover 24 kernel and 48 replay operations, 9,360 contact samples, both reference faces, reverse selections, transforms, topology, immutable inputs and STEP. Six native controls match analytic material within 5e-16 and exported Truck geometry within 2.8e-12 in sampled comparisons. One existing ignore and the known geometry precision failure remain explicit.

Clean capture: all 396 native cases pass across 648 operations. 450 constructions (+1) and 418 limited comparison passes (+0). All prior successful checks survive; 449 prior complete result/comparison records are exactly unchanged. 0 prior numerical changes. 94 reference samples were added; all other task fields, native references and selection scopes are unchanged. Of the 648 operations, 559 have valid solid inputs and native solid results; 88 have no solid input, and 1 is an expected native rejection that Truck also rejects.

Statuses: {"input-import-failed": 96, "operation-succeeded": 450, "operation-failed": 91, "unhandled-intent": 2, "input-inconsistent": 9}.

Status changes: [{"case": "chamfer/dist_dist_sequence/C1", "capture": "001", "changed_scope": false, "before": "unhandled-intent", "after": "operation-succeeded"}, {"case": "chamfer/dist_dist_sequence/C1", "capture": "002", "changed_scope": false, "before": "unhandled-intent", "after": "operation-failed"}, {"case": "chamfer/dist_dist_sequence/C1", "capture": "003", "changed_scope": false, "before": "unhandled-intent", "after": "operation-failed"}].

Archive SHA256: 207dc2fe168fe70b1d8fb03724c4772b31bbe7ad7fbd0d75ea4de7f5e5573318.

The new C1 unequal-distance result still differs from the native boundary; later captures fail at trimming. General curved angles and chord-contact continuation remain unsupported. No parity, global error bound or performance claim. No consumer changes, push or PR.
