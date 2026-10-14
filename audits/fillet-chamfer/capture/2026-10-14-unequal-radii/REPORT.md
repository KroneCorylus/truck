# Full capture after per-edge fillet radii

Kernel: `35c6e2cf3e562c1c587f0264d4c4e7dca1bb71ea`, from a clean tracked checkout. Source and executable hashes were checked before and after capture and while packaging. All 156 native entries passed, yielding the same 161 operation captures.

| Check | Previous | Current |
| --- | ---: | ---: |
| Constructed, unchanged input, consistent geometry | 101 | 104 |
| Closed Truck mesh | 101 | 104 |
| Native-valid single-solid STEP | 101 | 104 |
| Combined limited comparison pass at 0.001 | 92 | 95 |
| Operation failed | 48 | 53 |
| Unhandled intent | 8 | 0 |
| Input import failed | 3 | 3 |
| Incomplete capture | 1 | 1 |
| Input inconsistent / timeout / panic | 0 / 0 / 0 | 0 / 0 / 0 |

**All 101 preceding successful result/comparison records are exactly unchanged.** No previously passing check regresses. The only eight status changes are the previously unhandled unequal-radius requests:

- `draft/angle/A2`, `B5`, `C4` now construct and pass closed-mesh, native STEP and combined limited comparison checks.
- `blend/simple/A4`, `P9`, `blend/tolblend_simple/A2`, `feat/featlf/E2`, `E3` now reach the kernel and return `UnsupportedTopology`, preserving their inputs. Their selected edges meet at unsupported unequal-radius junctions. This is attempted coverage, not successful construction, and explains the operation-failure count rising by five.

The public solid API exposes per-edge radii on separate straight edges, reusing the existing constructor and retaining full history. Equal radii preserve the prior solid API behavior. Replay retains each captured radius and rejects conflicting physical-edge assignments. Independent material, normals/radii, transformed solids, selection order, immutable failure, parent failures, release tests and targeted native checks are recorded in `truck-benchmarks/results/2026-10-14-unequal-radii/`.

All **2,225** archive members were byte-verified. SHA-256: `7a5608fdb1e5ee58f7ffbf830b6a4fd9b8f1e8e73dbc3a554b547ecdfe21b1cd`.

Nine sampled discrepancies, three imports and one incomplete capture remain. Runtime coverage is still missing for 279 dedicated chamfer fixtures. General unequal-radius corners, unequal curved contours and global collisions are not added by this group. These are captured-subset results, not full parity, certified global bounds or a performance claim.
