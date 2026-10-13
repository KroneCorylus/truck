# Full capture after closed Bézier prism import recovery

Kernel: `fef68affc5e78615bdcc76e961c450b2fc4c5cbf`, from a clean tracked checkout. The production source and executable were frozen for the run; their hashes were verified before and after capture and again during packaging. All 156 native entries passed, producing 161 operation captures.

| Check | Previous capture | This capture |
| --- | ---: | ---: |
| Constructed with unchanged input and consistent geometry | 96 | 96 |
| Closed Truck mesh | 96 | 96 |
| Native-valid single-solid STEP export | 96 | 96 |
| Combined limited comparison pass at 0.001 | 87 | 87 |
| Operation failed | 48 | 53 |
| Input import failed | 8 | 3 |
| Input inconsistent | 0 | 0 |
| Unhandled intent | 8 | 8 |
| Incomplete capture | 1 | 1 |
| Timeout / panic | 0 / 0 | 0 / 0 |

**All 96 previous successful checks are retained, and all 96 STEP/sample comparison records remain exactly unchanged.** Of the complete Truck result records, 95 are unchanged. J9 alone changes its mesh volume from 1808712.393059274 to 1808714.8871952966 (a relative change of 1.379e-6), while retaining a closed mesh. Its native-valid STEP volume remains 1808730.6615535542; sampled maximum distances remain 9.419886071687929e-5 and 1.0987859336826511e-4. The ordered-boundary seed affects its tessellation; the unchanged imported-spline-pipe and both spline-pipe blend regressions pass. No previous successful check regresses. The only status changes are `bugs/modalg_7/bug30940_2` through `_6`: five closed Bézier-prism captures now import and reach their unsupported blend operations, preserving the input on failure. `_2` and `_5` reject nontangent rim wires; `_3` cannot resolve rolling contacts; `_4` and `_6` require nonplanar multi-edge corners. Input recovery is not a claim of five new fillet successes.

The implementation proof independently validates cubic and quartic input volumes 150 and 7200/7, closed meshes and native-valid single-solid STEP. See `truck-benchmarks/results/2026-10-13-closed-bezier-imports/REPORT.md` for parent failures, release tests, targeted replay, raw input provenance and independent checks.

All **2,201** archive members were byte-verified against their recorded hashes. Archive SHA-256:
`21903fc7962e6d62635b35dfcf70b89b915da7ea5d537df4ef4ec9c8ce71c749`.

Compared with the supplied 2026-10-12 seam-junction handoff, the same capture set now has 96 rather than 80 valid constructions, 87 rather than 78 combined limited comparison passes, three rather than 22 import failures, zero rather than 11 inconsistent inputs, and zero rather than two timeouts. There are still nine sampled discrepancies, eight unhandled unequal-radius intents, one incomplete capture, and no coverage of the missing 279 dedicated chamfer fixtures. These are subset results, not full parity or a performance claim.
