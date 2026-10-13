# Full captured comparison after spline support-domain repair

The complete available capture/replay suite ran at clean commit **8225327d**
against OpenCascade 7.9.3. All **156 native entries passed**, producing the
same **161 operation captures**. Source and executable hashes were unchanged
throughout the run and were verified before and after archiving.

| Outcome | Previous full run | This full run |
| --- | ---: | ---: |
| Constructed Truck output | 88 | **92** |
| Closed Truck mesh | 88 | **92** |
| Native-valid single-solid STEP output | 88 | **92** |
| Passed all limited comparison checks | 82 | **85** |
| Construction failed | 42 | 46 |
| Input import rejected | 11 | 11 |
| Inconsistent imported input | 11 | **3** |
| Request not expressed by replay | 8 | 8 |
| Incomplete native capture | 1 | 1 |
| Replay timed out | 0 | 0 |
| Panic | 0 | 0 |

The mesh, export and comparison counts are subsets of constructed outputs.
All 92 constructions preserve their inputs, pass geometric consistency, close
their meshes and export native-valid single-solid STEP. All **88 previously
successful operations retain exactly the same Truck result and complete
comparison record**. No previously successful check regressed.

## Spline prism recovery

Eight previously inconsistent inputs now pass import validation. G3, G5, G7
and G9 construct successfully. G3, G7 and G9 meet the 0.001 limited sampled
comparison threshold. G5 retains maximum sampled distances of approximately
**0.001707** and **0.001703**, so it is not counted as a comparison pass.

G4/G8 reach the existing nontangent-wire rejection; G6/H1 reach the existing
unsupported nonplanar-corner path. This explains the increased construction
failure count: those four requests now reach the blend operation. They
preserve their inputs on failure. The remaining inconsistent inputs are H6,
H7 and H9's conical sectors.

The original G1, tolerance-grid A1, H4 and K2/K3/K4 sampled discrepancies are
unchanged. `bugs/moddata_1/bug119` capture 001 remains incomplete. The missing
fixture corpus, including all 279 dedicated chamfer entries, is unchanged.
This captured subset does not establish overall fillet/chamfer parity.

## Evidence

All **2,189** archive members were byte-verified before and after creating
`artifacts.zip`. Its SHA-256 is
`1a7a96fd6509c05fe06f37002cc5f8ff741cf49ef65237de3715ba72d364d8b5`.
`results.json` records source/executable/runtime hashes, native execution,
physical selections and Truck/native results. `comparison-summary.json`
records the eight status changes, unchanged prior successes and no regressions.
The previous capture under `../2026-10-13-straight-end-extension/` is unchanged.
The regenerated catalog passed `--check`; all 19 audit tests and both capture
tests passed.

Production proof is in
`truck-benchmarks/results/2026-10-13-spline-sweep-bounds/REPORT.md`:
parent-failing regressions, exact independent input volumes, finite polynomial
and rational support bounds, 461 passing affected release tests, all-target
release Clippy and action benchmark smoke. No public API or dependency changed.

Reproduction from the repository root:

```sh
CARGO_BUILD_JOBS=12 cargo build --release -p truck-shapeops --example replay_occt_blend
python3 audits/fillet-chamfer/capture/run_capture.py \
  --occt ../investigations/sequential-holes-2026-10-10/occt \
  --library-path target/occt-sdk/usr/lib64 --plugin target/occt-capture-plugin \
  --replay target/release/examples/replay_occt_blend \
  --output target/NEW-CAPTURE-DIRECTORY --workers 4 --timeout 120
```
