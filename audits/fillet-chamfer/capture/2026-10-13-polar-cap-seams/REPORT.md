# Full captured comparison after polar cap repair

The complete available capture/replay suite ran at clean commit **1cb33dd8**
against OpenCascade 7.9.3. All **156 native entries passed**, producing the
same **161 operation captures**. Source and executable hashes were unchanged
throughout the run and were verified again before and after archiving.

| Outcome | Previous full run | This full run |
| --- | ---: | ---: |
| Constructed Truck output | 87 | 87 |
| Closed Truck mesh | 86 | **87** |
| Native-valid single-solid STEP output | 85 | **86** |
| Passed all limited comparison checks | 81 | 81 |
| Construction failed | 43 | 43 |
| Input import rejected | 11 | 11 |
| Inconsistent imported input | 11 | 11 |
| Request not expressed by replay | 8 | 8 |
| Incomplete native capture | 1 | 1 |
| Replay timed out | 0 | **0** |
| Panic | 0 | **0** |

The mesh, export and comparison counts are subsets of constructed outputs.
All 87 constructions preserve their inputs and pass Truck's geometric
consistency check. No previously successful check regressed.

## H4 now closes and exports valid STEP

The central-axis ellipsoid fillet has a closed Truck mesh and a native-valid
single-solid STEP result. Its exported face includes the previously missing
degenerate pole boundary. The modeling topology remains unchanged.

An independent corner-fill integral gives volume **13159710.462164925**;
native Gauss–Kronrod integration of the generated STEP gives
**13159710.46449488**. Its maximum native edge/vertex tolerance is **0.00001**.
Radius, normals, independent volume, topology, mesh closure, input immutability
and STEP regressions pass on both meridian sheets with reversed selections,
rigid motion and two scales.

H4 is still not counted as a sampled parity pass. Its captured native reference
has volume **13151259.698596016** and edge/vertex tolerances up to **22.9741
model units**, so native validity does not establish accurate material near
its pole. The sampled maximum distances remain approximately **0.00218** and
**4.14211** in the two directions. These disagreements are retained in the
manifest. The ordinary native STEP volume estimate in the comparator is
**13159710.046042992**; the independent diagnostic uses Gauss–Kronrod integration.

Every other successful capture retains **exactly the same Truck result and
complete comparison record**: 86 unchanged constructions, including all prior
limited comparison passes. H4 is the only operation with changed numeric or
boolean checks; no operation status changed.

G1, tolerance-grid A1 and K2/K3 retain their sampled discrepancies. K4 still
fails STEP preparation. `bugs/moddata_1/bug119` capture 001 remains incomplete.
The missing fixture corpus, including all 279 dedicated chamfer entries, is
unchanged. This captured subset does not establish overall fillet/chamfer parity.

## Evidence

`results.json` records source/executable/runtime hashes, native execution,
physical selections and Truck/native results. Every manifest-listed artifact
was verified before and after creating `artifacts.zip`, including all **2,171**
archive members. Its checksum is stored in the manifest.
`comparison-summary.json` records changed checks, unchanged statuses and the
86 unchanged prior outputs. The previous full run under
`../2026-10-13-ellipsoid-domains/` is unchanged.
The regenerated catalog passed `--check`; all 19 audit tests and both capture
tests passed.

Production evidence is in
`truck-benchmarks/results/2026-10-13-polar-cap-seams/REPORT.md`: parent-failing
regressions, 549 passing affected release tests, all-target release Clippy,
benchmark smoke and the independent native volume diagnostic. The STEP
constructor's new geometry trait bounds are documented in the Breaking
changelog entry; all PiezaCAD call sites already use the required concrete types.

Reproduction from the repository root:

```sh
CARGO_BUILD_JOBS=12 cargo build --release -p truck-shapeops --example replay_occt_blend
python3 audits/fillet-chamfer/capture/run_capture.py \
  --occt ../investigations/sequential-holes-2026-10-10/occt \
  --library-path target/occt-sdk/usr/lib64 --plugin target/occt-capture-plugin \
  --replay target/release/examples/replay_occt_blend \
  --output target/NEW-CAPTURE-DIRECTORY --workers 4 --timeout 120
```
