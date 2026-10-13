# Full captured comparison after exact diagonal pcurve conversion

The complete available capture/replay suite ran at clean commit **2978d651**
against OpenCascade 7.9.3. All **156 native entries passed**, producing the same
**161 operation captures** as the previous full run. Source and executable
hashes were unchanged throughout the run.

| Outcome | Previous full run | This full run |
| --- | ---: | ---: |
| Constructed Truck output | 80 | **85** |
| Passed all limited comparison checks | 78 | **80** |
| Construction failed | 37 | 43 |
| Input import rejected | 22 | **11** |
| Inconsistent imported input | 11 | 11 |
| Request not expressed by replay | 8 | 8 |
| Replay timed out | 2 | 2 |
| Panic | 0 | **0** |
| Incomplete native capture | 1 | 1 |

The limited comparison count is a subset of constructed outputs. All 85
constructed outputs preserve their inputs, pass geometric consistency and
have closed Truck meshes. **84 prepare STEP and produce native-valid single
solids; 80 also meet the 0.001 bidirectional sampled boundary-distance check.**

## Newly reachable operations

- `blend/simple/J1` and `bugs/heal/bug26244` construct, export and pass all
  limited checks. Their maximum sampled distances are below 0.00001.
- `blend/simple/K2` and `K3` construct and produce native-valid STEP solids,
  but differ from the reference by approximately 0.033 / 0.046 in the two
  sampling directions. Their exported volumes exceed the native references
  by approximately 24.93. These are newly measured discrepancies, not parity.
- `blend/simple/K4` constructs a consistent, closed solid but STEP preparation
  fails. No native comparison is available for this output.
- `blend/complex/G9` capture 002 and `bugs/modalg_5/bug25657_1` now reach the
  solver and return `BlendConstructionFailed` without changing their inputs.
- `feat/featrf/A4`, `A5`, `A6` and `B4` now import, but their four-edge
  selections still return `NonPlanarFace` without changing the inputs.

No previously successful output or check regressed. All 80 previous successful
captures retain exactly the same reported mesh/STEP volumes, sample counts
and maximum bidirectional sampled distances. Machine-readable changes and
counts are in `comparison-summary.json`.

## Remaining observations and scope

H2/H4 still exceed 120 seconds. The previous G1 and tolerance-grid A1 sampled
discrepancies remain unchanged. `bugs/moddata_1/bug119` capture 001 remains
incomplete. The catalog's missing fixture corpus, including all 279 dedicated
chamfer entries, is unchanged. Neither construction success nor these limited
sampled comparisons establishes full fillet/chamfer parity.

Every manifest-listed artifact was verified both before and after archiving.
`results.json` records the archive SHA-256, physical selections, native/Truck
results, source hashes and executable/runtime hashes. The previous full run
under `../2026-10-12-seam-junction/` is unchanged. Production regression and
validation evidence is in
`truck-benchmarks/results/2026-10-13-diagonal-pcurves/REPORT.md`.

Reproduction from the repository root:

```sh
CARGO_BUILD_JOBS=12 cargo build --release -p truck-shapeops --example replay_occt_blend
python3 audits/fillet-chamfer/capture/run_capture.py \
  --occt ../investigations/sequential-holes-2026-10-10/occt \
  --library-path target/occt-sdk/usr/lib64 --plugin target/occt-capture-plugin \
  --replay target/release/examples/replay_occt_blend \
  --output target/NEW-CAPTURE-DIRECTORY --workers 4 --timeout 120
```
