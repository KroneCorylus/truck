# Full capture after atomic separate-contour fillets

Kernel: `35d7304433f8aae964cb4ea1335b5b7c0bd50334`, from a clean tracked checkout. Source and replay executable hashes were checked before and after capture and again while packaging. All 156 native entries passed, producing the same 161 operation captures.

| Check | Previous cusp group | Current |
| --- | ---: | ---: |
| Constructed, unchanged input, consistent geometry | 97 | 101 |
| Closed Truck mesh | 97 | 101 |
| Native-valid single-solid STEP | 97 | 101 |
| Combined limited comparison pass at 0.001 | 88 | 92 |
| Operation failed | 52 | 48 |
| Input import failed | 3 | 3 |
| Unhandled intent | 8 | 8 |
| Incomplete capture | 1 | 1 |
| Input inconsistent / timeout / panic | 0 / 0 / 0 | 0 / 0 / 0 |

Only `feat/featrf/A4`, `A5`, `A6` and `B4` change status/checks: all four now construct and pass closure, native STEP validity and the combined limited comparison. They repeat one initial cylinder/cone-stack fillet before later rib features, so they establish one geometric capability. **All 97 previous successful result/comparison records are exactly unchanged.** No successful check regresses.

Independent material, radius/normals, transformed inputs, deterministic selections, complete history, atomic failure, parent regression and targeted native evidence are recorded in `truck-benchmarks/results/2026-10-14-separate-contours/`. Separate closed tangent contours require distinct support faces and cannot re-trim a common changed face. General corners, shared supports, collapse and distant-face collision handling are not added by this group.

All **2,216** archive members were byte-verified. SHA-256: `aa05a8d86cd34d02f5ac4120fcce84c59377ab27165de547a0b5dedc303d2e36`.

Nine sampled discrepancies remain, alongside three imports, eight unequal-radius intents, one incomplete capture and missing runtime coverage for 279 dedicated chamfer cases. This is evidence for a captured subset, not full parity or a performance claim.
