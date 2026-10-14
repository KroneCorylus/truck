# Full capture after cusp-contact and rolling-inverse improvements

Kernel: `0eaffdcd1b063d877c829f6bba7f26a66d766bbb`, from a clean tracked checkout. Source and replay executable hashes were checked before and after the run and again during packaging. All 156 native entries passed, producing 161 operation captures.

| Check | Previous | Current |
| --- | ---: | ---: |
| Constructed, unchanged input, consistent geometry | 96 | 97 |
| Closed Truck mesh | 96 | 97 |
| Native-valid single-solid STEP | 96 | 97 |
| Combined limited comparison pass at 0.001 | 87 | 88 |
| Operation failed | 53 | 52 |
| Input import failed | 3 | 3 |
| Unhandled intent | 8 | 8 |
| Incomplete capture | 1 | 1 |
| Input inconsistent / timeout / panic | 0 / 0 / 0 | 0 / 0 / 0 |

The only status/check change is `bugs/modalg_7/bug30940_3`, which now passes construction, closure, STEP and the combined limited comparison. **All 96 previous successful Truck results and comparison records are exactly unchanged.** No successful check regresses.

Independent material, radius, normals, transformed/reversed selections, parent failures, release tests and targeted native evidence are recorded in `truck-benchmarks/results/2026-10-14-cusp-contacts/`. This group includes support-branch propagation, scale-consistent rolling inverses and safe singular inverse failures. It does not claim support for nontangent rims, nonplanar multi-edge corners or rolling-support collapse.

All **2,204** archive members were byte-verified against their recorded hashes. Archive SHA-256: `c9ab56e2a9a948a3e7a786d5afa3567fb08423c43fbca435beca223c822274ba`.

Nine sampled discrepancies remain. Three imports, eight unequal-radius intents, one incomplete capture and the missing 279 dedicated chamfer fixture cases remain open. These results describe the captured subset, not full parity or a performance improvement.
