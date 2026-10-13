# Full captured comparison after exact cubic endpoint repair

The complete available capture/replay suite ran at clean commit **78baa696**
against OpenCascade 7.9.3. All **156 native entries passed**, producing the
same **161 operation captures**. Source and executable hashes were unchanged
throughout the run and were verified before and after archiving.

| Outcome | Previous full run | This full run |
| --- | ---: | ---: |
| Constructed Truck output | 87 | 87 |
| Closed Truck mesh | 87 | 87 |
| Native-valid single-solid STEP output | 86 | **87** |
| Passed all limited comparison checks | 81 | 81 |
| Construction failed | 43 | 43 |
| Input import rejected | 11 | 11 |
| Inconsistent imported input | 11 | 11 |
| Request not expressed by replay | 8 | 8 |
| Incomplete native capture | 1 | 1 |
| Replay timed out | 0 | 0 |
| Panic | 0 | 0 |

The mesh, export and comparison counts are subsets of constructed outputs.
All 87 constructions preserve their inputs and pass Truck's geometric
consistency check. Every constructed result now has both a closed mesh and a
native-valid single-solid STEP export. No previously successful check regressed.

## K4 STEP recovery

The cylindrical junction now exports successfully. Its modeling result and
closed mesh are unchanged. Native STEP volume is **214210.88280914773**;
the native reference reports **214190.18597261963**. Maximum sampled distances
are **0.04617208169705975** and **0.05617813045504337**. The discrepancy remains
recorded, so K4 is not counted as a limited comparison pass.

Every other successful capture retains exactly the same Truck result and
complete comparison record: **86 unchanged constructions**, including all
prior limited comparison passes. K4 is the only operation with changed numeric
or boolean checks. No operation status changed.

The independent cylinder-offset diagnostic in the production evidence finds
a maximum sampled radius/normal residual of **0.00037817** for Truck K4 versus
**0.07369132** for the native reference. This supports the sampled geometric
quality of the export but does not establish a certified global error bound.

G1, tolerance-grid A1, H4 and K2/K3 retain their sampled discrepancies.
`bugs/moddata_1/bug119` capture 001 remains incomplete. The missing fixture
corpus, including all 279 dedicated chamfer entries, is unchanged. This captured
subset does not establish overall fillet/chamfer parity.

## Evidence

`results.json` records source/executable/runtime hashes, native execution,
physical selections and Truck/native results. Every manifest-listed artifact
was verified before and after creating `artifacts.zip`, including all **2,174**
archive members. The archive SHA-256 is
`fe9df40abe48d8387b6ccf721f48a3c75a31cd6c057dccbf7654bfef36a66e9a`.
`comparison-summary.json` records changed checks, unchanged statuses and the
86 unchanged prior outputs. The previous full run under
`../2026-10-13-polar-cap-seams/` is unchanged.
The regenerated catalog passed `--check`; all 19 audit tests and both capture
tests passed.

Production evidence is in
`truck-benchmarks/results/2026-10-13-cubic-endpoints/REPORT.md`: parent-failing
regressions, 912 passing affected release tests, the one reproduced unchanged
PCurve precision failure, all-target release Clippy, benchmark smoke and the
independent cylinder-offset diagnostic. No performance claim is made.

Reproduction from the repository root:

```sh
CARGO_BUILD_JOBS=12 cargo build --release -p truck-shapeops --example replay_occt_blend
python3 audits/fillet-chamfer/capture/run_capture.py \
  --occt ../investigations/sequential-holes-2026-10-10/occt \
  --library-path target/occt-sdk/usr/lib64 --plugin target/occt-capture-plugin \
  --replay target/release/examples/replay_occt_blend \
  --output target/NEW-CAPTURE-DIRECTORY --workers 4 --timeout 120
```
