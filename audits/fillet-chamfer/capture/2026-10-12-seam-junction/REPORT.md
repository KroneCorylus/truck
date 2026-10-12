# Full captured comparison after the seam-junction fix

The complete available capture/replay suite ran at clean commit **946914f8**,
whose kernel is **f6fa8d15**, against OpenCascade 7.9.3. All **156 native test
entries passed**. They produced **161 operation captures**, matching the previous
run's scope. No kernel source or replay executable changed during execution.

| Outcome | Previous full run | This full run |
| --- | ---: | ---: |
| Constructed Truck output | 80 | 80 |
| Passed all limited comparison checks | 78 | 78 |
| Construction failed | 37 | 37 |
| Input import rejected | 21 | 22 |
| Inconsistent imported input | 11 | 11 |
| Request not expressed by replay | 8 | 8 |
| Replay timed out | 2 | 2 |
| Panic | 1 | **0** |
| Incomplete native capture | 1 | 1 |

The limited comparison count is a subset of constructed outputs, not an
additional outcome. All 80 constructed outputs preserve their inputs, pass
geometric consistency, have closed Truck meshes, prepare STEP and produce
native-valid single solids. Seventy-eight also have sampled boundary distances
no greater than 0.001 in both directions.

## Change from the previous run

**G9 capture 002 is the only operation-status change.** It previously panicked
in closed-face preprocessing. It now returns `input-import-failed` with
`Unsupported imported geometry`. Preprocessing no longer crashes; conversion
of a generated surface curve remains unsupported, so the requested fillet is
still unresolved.

No previously successful construction or limited check regressed. All 80
constructed cases retain exactly the same reported mesh volumes, exported
STEP volumes and maximum bidirectional sampled distances as the previous run.
The machine-readable differences and check counts are in comparison-summary.json.

## Remaining observations

- `blend/simple/H2` and `H4` still exceed the 120-second replay limit.
- `blend/simple/G1` retains maximum distances 0.9874986975 / 0.01854574905.
- `blend/tolblend_simple/A1` retains maximum distances 0.02237647257 in both directions.
- `bugs/moddata_1/bug119` capture 001 remains incomplete.
- The other construction/import/inconsistent-input/unhandled-request outcomes
  are unchanged. This rerun adds validation evidence, not new geometry features.

## Scope and evidence

This is the full available **captured blend comparison**, not execution of every
catalog entry or every chamfer feature. The separate catalog still includes 701
runtime-skipped entries because fixtures are unavailable, including its 279
dedicated chamfer cases. The replay does not express every OCCT blend/chamfer intent.
Sampled agreement does not establish complete feature parity, global collision
freedom, G1 continuity, history behavior or performance.

The manifest records the clean revision, source/executable/runtime hashes,
physical selections, native and Truck results. Every manifest-listed artifact
was hash-verified before archiving; the archive hash is retained in results.json.
The previous full run remains unchanged in `../2026-10-12-periodic-tori/`.

Reproduction from the repository root:

```sh
CARGO_BUILD_JOBS=12 cargo build --release -p truck-shapeops --example replay_occt_blend
python3 audits/fillet-chamfer/capture/run_capture.py \
  --occt ../investigations/sequential-holes-2026-10-10/occt \
  --library-path target/occt-sdk/usr/lib64 --plugin target/occt-capture-plugin \
  --replay target/release/examples/replay_occt_blend \
  --output target/NEW-CAPTURE-DIRECTORY --workers 4 --timeout 120
```
