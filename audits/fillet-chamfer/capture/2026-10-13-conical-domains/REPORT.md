# Full captured comparison after conical support-domain repair

The complete available capture/replay suite ran at clean commit **2fa70c58**
against OpenCascade 7.9.3. All **156 native entries passed**, producing the same
**161 operation captures**. Source and executable hashes were unchanged through
the run and verified before and after archiving.

| Outcome | Previous full run | This full run |
| --- | ---: | ---: |
| Constructed Truck output | 92 | **93** |
| Closed Truck mesh | 92 | **93** |
| Native-valid single-solid STEP output | 92 | **93** |
| Passed all limited comparison checks | 85 | **86** |
| Construction failed | 46 | 48 |
| Input import rejected | 11 | 11 |
| Inconsistent imported input | 3 | **0** |
| Request not expressed by replay | 8 | 8 |
| Incomplete native capture | 1 | 1 |
| Replay timed out | 0 | 0 |
| Panic | 0 | 0 |

The mesh, export and comparison counts are subsets of constructed outputs.
All 93 constructions preserve their inputs, pass geometric consistency, close
their meshes and export native-valid single-solid STEP. No previously passing
check regressed.

H6's conical-sector neck fillet now constructs and meets the 0.001 limited
sampled comparison threshold: the two maximum distances are approximately
0.000019102. H7/H9 also import consistently, then reach the existing unsupported
three-edge nonplanar-corner path. Their failures preserve the input. This
accounts for the increase in operation failures.

Of the 92 earlier successes, **81 retain exactly the same complete Truck result
and comparison records**. Eleven have numerical changes after cone support
reparameterization, recorded individually in `comparison-summary.json`. The
largest mesh-volume change is about 0.009491 (I1); the largest native-integrated
Truck STEP volume change is about 0.00002185 (J1). I3's two sampled distances
improve from approximately 0.00001139/0.00000843 to
0.00000394/0.00000398. All eleven retain their previous passing checks.

G1, tolerance-grid A1, H4, K2/K3/K4 and G5 retain their sampled discrepancies.
`bugs/moddata_1/bug119` capture 001 remains incomplete. The missing fixture
corpus, including all 279 dedicated chamfer entries, is unchanged. The regenerated catalog passed `--check`; all 19 audit tests
and both capture tests passed. This captured
subset does not establish overall fillet/chamfer parity.

All **2,192** archive members were byte-verified before and after creating
`artifacts.zip`. Its SHA-256 is
`f1d7b5780e586ba4eec326be606d273b687d27cb419b4565f6547c94569a2afc`.
`results.json` records source/executable/runtime hashes, physical selections and
raw outcomes. The previous capture under `../2026-10-13-cone-pole-meshing/`
is unchanged. The regenerated catalog passed `--check`; all 19 audit tests
and both capture tests passed.

Production proof is in
`truck-benchmarks/results/2026-10-13-conical-domains/REPORT.md`: parent-failing
bounds/export/solid regressions, independent input and fillet volumes, 466
passing affected release tests, all-target release Clippy and action benchmark
smoke. No public API or dependency changed; no performance claim is made.

Reproduction from the repository root:

```sh
CARGO_BUILD_JOBS=12 cargo build --release -p truck-shapeops --example replay_occt_blend
python3 audits/fillet-chamfer/capture/run_capture.py \
  --occt ../investigations/sequential-holes-2026-10-10/occt \
  --library-path target/occt-sdk/usr/lib64 --plugin target/occt-capture-plugin \
  --replay target/release/examples/replay_occt_blend \
  --output target/NEW-CAPTURE-DIRECTORY --workers 4 --timeout 120
```
