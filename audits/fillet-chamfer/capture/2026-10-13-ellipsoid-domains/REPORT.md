# Full captured comparison after elliptical revolution domain repair

The complete available capture/replay suite ran at clean commit **2b25a964**
against OpenCascade 7.9.3. All **156 native entries passed**, producing the
same **161 operation captures**. Source and executable hashes were unchanged
throughout the run and were verified again after archiving.

| Outcome | Previous full run | This full run |
| --- | ---: | ---: |
| Constructed Truck output | 85 | **87** |
| Closed Truck mesh | 85 | **86** |
| Native-valid single-solid STEP output | 84 | **85** |
| Passed all limited comparison checks | 80 | **81** |
| Construction failed | 43 | 43 |
| Input import rejected | 11 | 11 |
| Inconsistent imported input | 11 | 11 |
| Request not expressed by replay | 8 | 8 |
| Replay timed out | 2 | **0** |
| Panic | 0 | **0** |
| Incomplete native capture | 1 | 1 |

The mesh, export and limited comparison counts are subsets of constructed
outputs. All 87 constructions preserve their inputs and pass Truck's geometric
consistency check; those checks alone do not establish valid material.

## Changed operations

- **H2** now constructs a closed solid and exports a native-valid single-solid
  STEP result. The bidirectional sampled maximum distances are approximately
  0.00001404 and 0.00001410, below the 0.001 comparison tolerance. Its independent
  radius, normal, contact-height and volume regressions pass on both meridian
  sheets, including rigid motion and reversed selection.
- **H4** now completes construction/export within the replay limit, but its
  Truck mesh is open and its STEP result fails native validity. This is newly
  exposed invalid output, not a recovered fillet capability. Its native
  reference also carries edge/vertex tolerances of approximately 22.9741 model
  units; native validity alone is insufficient evidence of accurate material
  near its pole. The diagnostic is preserved with the production evidence.

Every previous successful capture retains **exactly the same Truck result and
complete sampled comparison record**, including volumes, sample counts and
maximum distances. No previously successful check regressed.

G1, tolerance-grid A1 and K2/K3 retain their sampled discrepancies. K4 still
fails STEP preparation. `bugs/moddata_1/bug119` capture 001 remains incomplete.
The missing fixture corpus, including all 279 dedicated chamfer entries, is
unchanged. This captured subset does not establish overall fillet/chamfer parity.

## Evidence

`results.json` records source/executable/runtime hashes, native execution,
physical selections and Truck/native results. Every manifest-listed artifact
was verified before and after creating `artifacts.zip`; the archive checksum
is stored in the manifest. All 2,171 archived files were verified. The catalog
was regenerated and passed `--check`; all 19 audit tests and two capture tests
passed. `comparison-summary.json` records status changes,
check counts and the unchanged prior outputs. The previous full run under
`../2026-10-13-diagonal-pcurves/` is unchanged.

Production regressions, independent volume derivations, targeted comparisons,
the known unchanged PCurve precision-test failure, final Clippy and benchmark
smoke evidence are in
`truck-benchmarks/results/2026-10-13-ellipsoid-domains/REPORT.md`.

Reproduction from the repository root:

```sh
CARGO_BUILD_JOBS=12 cargo build --release -p truck-shapeops --example replay_occt_blend
python3 audits/fillet-chamfer/capture/run_capture.py \
  --occt ../investigations/sequential-holes-2026-10-10/occt \
  --library-path target/occt-sdk/usr/lib64 --plugin target/occt-capture-plugin \
  --replay target/release/examples/replay_occt_blend \
  --output target/NEW-CAPTURE-DIRECTORY --workers 4 --timeout 120
```
