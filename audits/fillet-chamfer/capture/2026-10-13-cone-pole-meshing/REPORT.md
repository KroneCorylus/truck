# Full captured comparison after conical pole meshing repair

The complete available capture/replay suite ran at clean commit **22b17350**
against OpenCascade 7.9.3. All **156 native entries passed**, producing the same
**161 operation captures**. Source and executable hashes were unchanged through
the run and were verified before and after archiving.

All **92 previous constructions retain exactly the same complete Truck result
and native comparison records**. They preserve their inputs, pass geometric
consistency, close their meshes and export native-valid single-solid STEP.
**85** meet all limited sampled comparison checks at tolerance 0.001. There are
no changed statuses, checks or numerical results and no regressions.

The remaining outcomes are unchanged: 46 operation failures, 11 rejected imports,
3 inconsistent conical inputs (H6/H7/H9), 8 requests not expressed by this replay,
and 1 incomplete capture. There are **zero timeouts and zero panics**. The cone
support-domain prototype is not part of this production run.

G1, tolerance-grid A1, H4, K2/K3/K4 and G5 retain their sampled discrepancies.
`bugs/moddata_1/bug119` capture 001 remains incomplete. The missing fixture
corpus, including all 279 dedicated chamfer entries, is unchanged. This captured
subset does not establish overall fillet/chamfer parity.

All **2,189** archive members were byte-verified before and after creating
`artifacts.zip`. Its SHA-256 is
`d0d74a8b900938f7c8f31dd826b89df181fa04ae13730497984e1a5df213fa0a`.
`results.json` records source/executable/runtime hashes and raw outcomes.
`comparison-summary.json` compares every capture with the preceding full run
under `../2026-10-13-spline-sweep-bounds/`, which is unchanged. The regenerated catalog passed `--check`; all 19 audit
tests and both capture tests passed.

Production proof is in
`truck-benchmarks/results/2026-10-13-cone-pole-meshing/REPORT.md`: both new
regressions fail on the parent; the five affected release suites record 921
passing tests, one unchanged PCurve precision failure and five existing ignored
tests. All-target release Clippy, action benchmark smoke and the final replay
build pass. No public API or dependency changed; no performance claim is made.

Reproduction from the repository root:

```sh
CARGO_BUILD_JOBS=12 cargo build --release -p truck-shapeops --example replay_occt_blend
python3 audits/fillet-chamfer/capture/run_capture.py \
  --occt ../investigations/sequential-holes-2026-10-10/occt \
  --library-path target/occt-sdk/usr/lib64 --plugin target/occt-capture-plugin \
  --replay target/release/examples/replay_occt_blend \
  --output target/NEW-CAPTURE-DIRECTORY --workers 4 --timeout 120
```
