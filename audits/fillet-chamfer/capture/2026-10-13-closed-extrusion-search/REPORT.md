# Full capture after closed extrusion inverse-search correction

Kernel: `cf16b061dad663d384dd70ab2346df1bba770bff`, from a clean tracked checkout. Source and executable hashes were verified before and after capture and again while packaging. All 156 native entries passed, producing 161 operation captures.

| Check | Previous capture | This capture |
| --- | ---: | ---: |
| Constructed with unchanged input and consistent geometry | 96 | 96 |
| Closed Truck mesh | 96 | 96 |
| Native-valid single-solid STEP export | 96 | 96 |
| Combined limited comparison pass at 0.001 | 87 | 87 |
| Operation failed | 48 | 48 |
| Input import failed | 8 | 8 |
| Input inconsistent | 0 | 0 |
| Unhandled intent | 8 | 8 |
| Incomplete capture | 1 | 1 |
| Timeout / panic | 0 / 0 | 0 / 0 |

**All 96 previous successful Truck results and sampled comparison records are exactly unchanged.** Every capture status and check is unchanged, with no numeric regressions. Closed Bézier-prism boundary reconstruction still fails safely during import; the inverse-search correction is independently covered by the new parent-failing geometry regressions in `truck-benchmarks/results/2026-10-13-closed-extrusion-search/`.

All **2,201** archive members were byte-verified against their recorded hashes. Archive SHA-256:
`d72e6745454c88d70fbefb93f85d4679522f01ccffd28d3352f3ff068f1e34ae`.

These checks cover the captured subset. They do not establish full parity, resolve the nine sampled discrepancies, or fill the missing dedicated chamfer fixture corpus. No performance claim is made.
