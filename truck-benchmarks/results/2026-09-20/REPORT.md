# Fillet surface search reuse

The threaded box and lid rebuild dropped from **17.064 s to 8.128 s** (52.4% less time).
The rolling-fillet kernel workload dropped from **191 ms to 15.69 ms** (12.2 times faster).
These are medians from the same machine and settings, not general speed guarantees.

## Change

`ApproxFilletSurface` previously rebuilt a rational Bezier cross-section at every point in
its 51 × 51 search grid. The new implementation constructs the 51 distinct cross-sections
once per search, then evaluates the same 2,601 grid points. It preserves sample coordinates,
iteration order, distance comparisons, tie handling, Newton refinement and tolerances.
There is no persistent cache or change to public APIs.

GDB stack sampling found this repeated work both in blend validation and in the application's
face identification after Boolean operations. Removing it therefore also improves the holes
and subsequent latch union, even though their own construction algorithms did not change.

## Measurements

| Workload | Before | After | Samples per version |
| --- | ---: | ---: | ---: |
| Kernel rolling fillet | 191 ms | 15.69 ms | 20 |
| Complete part build, including display meshes | 17.064 s | 8.128 s | 3 |
| Wall corner fillets | 4.126 s | 0.595 s | 3 |
| Four modeled M8 × 1.25 holes | 7.095 s | 4.310 s | 3 |
| Front latch catch union | 5.542 s | 3.129 s | 3 |

The part is `models/box-100mm/box-and-lid.json`: 18 features, two bodies, outside floor
chamfers, wall fillets, four modeled threaded holes and a PLA latch. Each sample reconstructs
its features in order with `build::resume`, including mesh construction; it excludes JSON
reading and process startup. Per-feature times include application face naming and display
work, so they are not isolated kernel Boolean timings. Samples use a warmed process without
reusing prior samples' built geometry.

An isolated Wayland application test also opened `box-and-lid-assembled.json` in **8.001 s**.
During loading, it completed 56 presentation probes and 57 state queries; the slowest state
query took 18.622 ms. This is a single end-to-end validation run using the test environment's
default Rayon worker count, separate from the four-worker medians above.

Machine: AMD Ryzen 9 7950X3D, Rust 1.98.1, Linux, optimized release/bench builds, system
allocator, powersave governor with boost, no CPU affinity pinning. Both median comparisons
use `RAYON_NUM_THREADS=4`. Baseline kernel commit: `8e385827`; baseline app: `c447fde`.
The after version adds only this production kernel change. Compilation finished before
timing. Exact revisions, input SHA-256 hashes, per-feature medians and the UI result are in
[`summary.json`](summary.json). Raw samples and Divan output are alongside this report.

## Reproduce

From the kernel repository:

```sh
RAYON_NUM_THREADS=4 cargo bench -p truck-benchmarks --bench actions -- rolling_fillet
```

From the sibling PiezaCAD repository, using its new `profile_document` example:

```sh
cargo build --release -p cad-core --example profile_document
RAYON_NUM_THREADS=4 target/release/examples/profile_document \
  ../models/box-100mm/box-and-lid.json 3

PIEZA_LOAD_DOCUMENT="$PWD/../models/box-100mm/box-and-lid-assembled.json" \
PIEZA_QA_DIR=/tmp/pieza-kernel-load \
scripts/headless-test.sh --release -p cad-app --test control \
  document_load_responsiveness_benchmark -- --ignored --nocapture
```

To reproduce the before comparison, use the baseline production source with the same
benchmark and profiling helper. The external user models are identified by hash; they are
not fixtures embedded in this kernel repository. Use the part document for the incremental
helper; the application test exercises assembly loading and source reuse.

## Correctness

- Exact equality between old and new search seeds and refined results for full, restricted,
  reversed and degenerate parameter ranges, with boundary, interior and off-surface points.
- A support-evaluation regression fails before the change (15,618 evaluations) and passes
  after, covering both exact and nearest point searches.
- All six part builds have identical body statistics, including volume and topology counts.
- `cargo test --release -p truck-geometry -p truck-shapeops`: 500 passed.
- `cargo test --release -p cad-core --lib`: 212 passed.
- All benchmark preflights pass with one Rayon worker, including closed-mesh checks.
- Clippy passes for geometry, modeling and shapeops across all targets.
- The isolated application load test passes with all features successful and ongoing UI
  responses throughout loading.

## Remaining cost

Eight seconds still leaves room for improvement. Post-change stack samples point to exact
NURBS surface searches during the application's Boolean face ownership identification
(`name_by_owner`). A control-point bounds shortcut was considered but not implemented:
kernel parameter searches support extrapolation, so rejecting points outside the ordinary
surface bounds could change valid results. A further optimization needs to preserve that
behavior or use a narrower contract at the caller.
