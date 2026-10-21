# Native distance correction: clean capture

Implementation 5fe64d3337b2a2f327c22945beea146ebc2972e5.

Corrected the native comparison tool when gradient search misses the nearest point on a freeform face. It now considers additional tree-search projections classified against the original trimmed face. Copying target topology also prevents measurement from changing a face flag. Sampling points, thresholds, operation requests, source geometry and volume integration are unchanged.

The unchanged regression fails three of 24 checks on the committed parent and passes all 24 with the correction. It covers exact surface subsets, transforms, trimmed boundaries, nested and separated solids, and byte-for-byte input preservation. Native volume (27), reference capture (8), capture harness (13) and catalog (22) checks pass. The Rust sources, geometry fixtures and rebuilt replay executable have matching hashes with the prior checkpoint: its 448 shapeops, 115 stepio and 64 replay tests, all-target Clippy and single-thread actions smoke remain applicable. The known geometry precision failure and existing ignore remain explicit.

All 396 native cases pass across 648 operations. Truck still constructs 450; every Truck result, task and native reference is unchanged. 418 now pass the limited comparisons (+0). 279 complete successful result/comparison records are exactly unchanged; 171 have changed distance measurements only. No previous check regresses. These are measurement corrections, not new Truck constructions.

Statuses: {"input-import-failed": 96, "operation-succeeded": 450, "operation-failed": 91, "unhandled-intent": 2, "input-inconsistent": 9}.

New limited comparison passes: [].

141 of 559 positive solid operations remain unresolved; 32 are constructed discrepancies.

Archive SHA256: 244426356bf5b1cf17363425b3f697dd3a53a0384fdeab06a2b81362787898fc.

Finite sampling does not prove a global distance bound or full parity. C1 still differs from the native boundary. No consumer/pin changes, push, PR or performance claim.
