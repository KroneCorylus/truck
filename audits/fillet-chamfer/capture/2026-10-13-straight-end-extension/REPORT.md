# Full captured comparison after straight end-boundary extension

The complete available capture/replay suite ran at clean commit **bbb3ebb0**
against OpenCascade 7.9.3. All **156 native entries passed**, producing the
same **161 operation captures**. Source and executable hashes were unchanged
throughout the run and were verified before and after archiving.

| Outcome | Previous full run | This full run |
| --- | ---: | ---: |
| Constructed Truck output | 87 | **88** |
| Closed Truck mesh | 87 | **88** |
| Native-valid single-solid STEP output | 87 | **88** |
| Passed all limited comparison checks | 81 | **82** |
| Construction failed | 43 | **42** |
| Input import rejected | 11 | 11 |
| Inconsistent imported input | 11 | 11 |
| Request not expressed by replay | 8 | 8 |
| Incomplete native capture | 1 | 1 |
| Replay timed out | 0 | 0 |
| Panic | 0 | 0 |

The mesh, export and comparison counts are subsets of constructed outputs.
All 88 constructions preserve their inputs, pass geometric consistency, close
their meshes and export native-valid single-solid STEP. No previously
successful check regressed.

## E9 recovered

E9's torus meridian fillet now extends its exact straight end boundary to the
reentrant contact. Maximum sampled distances are approximately **0.00004165**
in both directions, within the 0.001 comparator threshold. Its independent
rolling-sphere surface integral gives volume **1537066.4574334237**; native
Gauss–Kronrod integration of the Truck STEP gives **1537066.4526492232**.
Maximum native edge/vertex tolerance is **0.000008234261041079063**.

All **87 previously successful operations retain exactly the same Truck result
and complete comparison record**. E9 is the only operation with changed
status or checks. G1, tolerance-grid A1, H4 and K2/K3/K4 retain their sampled
discrepancies. `bugs/moddata_1/bug119` capture 001 remains incomplete.
The missing fixture corpus, including all 279 dedicated chamfer entries, is
unchanged. This captured subset does not establish overall fillet/chamfer parity.

## Evidence

`results.json` records source/executable/runtime hashes, native execution,
physical selections and Truck/native results. All **2,177** archive members
were byte-verified before and after creating `artifacts.zip`. Its SHA-256 is
`f7fb67e4292d11ceb61d4aff5d2054052283abd7129d09a84dcbd742d2d907a9`.
`comparison-summary.json` records the E9 recovery, unchanged prior successes
and absence of regressions. The previous full capture under
`../2026-10-13-cubic-endpoints/` is unchanged.
The regenerated catalog passed `--check`; all 19 audit tests and both capture
tests passed.

Production proof is in
`truck-benchmarks/results/2026-10-13-straight-end-extension/REPORT.md`:
parent-failing regressions, independent geometry and volume, 358 passing
release tests, all-target release Clippy and action benchmark smoke. The
generic chain APIs' exact-line conversion bound is documented in the Breaking
changelog entry; PiezaCAD's modeling-solid call needs no migration.

Reproduction from the repository root:

```sh
CARGO_BUILD_JOBS=12 cargo build --release -p truck-shapeops --example replay_occt_blend
python3 audits/fillet-chamfer/capture/run_capture.py \
  --occt ../investigations/sequential-holes-2026-10-10/occt \
  --library-path target/occt-sdk/usr/lib64 --plugin target/occt-capture-plugin \
  --replay target/release/examples/replay_occt_blend \
  --output target/NEW-CAPTURE-DIRECTORY --workers 4 --timeout 120
```
