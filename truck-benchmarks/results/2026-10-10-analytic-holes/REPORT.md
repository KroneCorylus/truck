# Analytic cylindrical hole subtraction

On the 100-hole sequential plate benchmark, one-worker Truck falls from **5,440.23 ms to 32.89 ms**, a **165.4× speedup**. Direct OCCT 7.9.3 takes **696.68 ms** in the same run, so optimized Truck is **21.2× faster on this workload**. With four workers, Truck falls from **2,942.72 ms to 32.88 ms**. This is a specialized improvement for isolated cylindrical through-holes; it is not a general Boolean performance claim.

## Sequential hole results

Median operation latency in milliseconds, calculated as the median of two round medians. Each round contains five samples after a warmup.

| Holes | Workers | Truck before | Truck after | Direct OCCT | Truck speedup |
|---:|---:|---:|---:|---:|---:|
| 1 | 1 | 1.520 | 0.023 | 1.368 | 66.8× |
| 10 | 1 | 62.273 | 0.488 | 18.306 | 127.6× |
| 30 | 1 | 461.111 | 3.405 | 88.769 | 135.4× |
| 100 | 1 | 5440.234 | 32.891 | 696.682 | 165.4× |
| 1 | 4 | 1.213 | 0.025 | 1.232 | 49.1× |
| 10 | 4 | 37.359 | 0.492 | 16.440 | 75.9× |
| 30 | 4 | 254.997 | 3.440 | 75.219 | 74.1× |
| 100 | 4 | 2942.724 | 32.884 | 577.988 | 89.5× |

The optimized path is serial, so increasing the worker limit does not improve it. The 100-hole Truck result has 206 faces; default OCCT uses one cylindrical wall per hole and has 106. Both represent the same stock and cuts. No artificial cylinder splitting is enabled for OCCT here.

## Adjacent operations

One worker, same timing protocol. Batch subtraction, filleting, meshing and knurling are within 1% of their parent medians in this run. The thin-wall bore also qualifies for the analytic path.

| Case | Truck before ms | Truck after ms |
|---|---:|---:|
| `holes_batch/100` | 184.838 | 183.888 |
| `thin_wall/0.01` | 8.629 | 0.018 |
| `fillet_bore/0.5` | 10.493 | 10.467 |
| `mesh_plate/100` | 62.476 | 62.052 |
| `knurl/16` | 9.447 | 9.390 |

## Implementation and limits

The subtraction APIs recognize a complete native circular extrusion that crosses both planar caps of a single prismatic body. Exact line/arc distance tests establish clearance from every cap boundary, and winding determines whether the cutter lies in material or empty space. Circular boundary chords used for winding have a bounded deviation smaller than the established clearance.

For an eligible cut, the existing plane/cylinder intersection helper supplies an exact circular section. The operation replaces the two cap faces, adds the cylindrical hole walls, and retains every other face and its identity. It performs no Boolean triangulation, pair search or global surface refitting. A cutter wholly inside existing empty space returns the unchanged target with `removed_material = false`.

Contacts, crossing or overlapping holes, blind cuts, multiple shells, cavities, oblique cuts and unsupported representations use the unchanged general solver. Recognition currently supports planar walls and native circular extrusion walls parallel to the cutter; cap boundaries must be lines or round conics. STEP/NURBS cylinders and revolved representations are not accelerated. Angular recognition also has a positional error bound; uncertain or numerically extreme cases fall back.

The four subtraction functions now require `'static` geometry types for safe concrete-type dispatch. Generic wrappers need the same bounds; PiezaCAD's owned modeling types need no call-site changes. There are no new dependencies, global caches or unsafe code. The application dependency pin has not changed.

## Correctness and validation

All **20 unchanged comparison preflights** pass. All **113 preflight/timed process records** pass the harness: solid outputs satisfy native geometry and topology checks, and all outputs satisfy mesh closure and the analytic volume bound of 0.2%. Timed results from different engines also agree on volume within that bound. The 100-hole optimized mesh has 29,212 triangles and 0.0310% volume error at 0.001 mm deflection; validation remains outside the operation timer. Mesh-volume error was 0.00981% before and is 0.0161% for OCCT. Native conic parameterization changes the tessellation, while the requested deflection and acceptance bound remain unchanged.

Nine new tests cover face retention and exact rims, transformed sequences, reversed axes and parameters, six-part circles, a 0.01 mm wall, empty-space cuts, concave outlines, remote plane parameter origins, STEP export, subsequent overlapping/blind cuts, input immutability and fallback boundaries. The face-retention regression fails on the parent: it retains zero original faces instead of four.

- `RAYON_NUM_THREADS=1 cargo test --release --locked -p truck-shapeops --no-fail-fast`: **276 passed, 1 failed, 1 existing ignored test**. The failure is `r11_corners::unsupported_junctions_have_distinct_diagnostics`, whose chamfer assertion expects an error but receives success. The identical failure was reproduced on the unchanged parent; its test was not modified. See [full tests](validation-tests.log) and [parent reproduction](validation-parent.log).
- `cargo clippy --release --locked -p truck-shapeops --all-targets`: passes.
- `RAYON_NUM_THREADS=1 cargo bench --locked -p truck-benchmarks --bench actions -- --test`: passes.
- Nightly rustfmt checks on changed Rust files and `git diff --check`: pass.

## Measurement and reproduction

Date: 2026-10-10. AMD Ryzen 9 7950X3D; every process pinned to CPUs 0–3. Worker limits are 1 and 4, with serial OCCT Booleans at one worker. Rust release builds use rustc 1.98.1. Truck parent: `3729a083932f5d73a29a4e84cc0b03866869aa27`; optimized source is that parent plus [source.patch](source.patch). OCCT is the source-built `V7_9_3`, commit `a016080bf6738d6aeae020badee4e888ad1540a5`, from the preceding investigation, with profiling and split-cylinder modes disabled.

The Rust comparison example and all fixtures are unchanged. Fixture construction, process startup, final-output destruction and output QA are outside the timer; intermediate Boolean work is inside. Operation tolerance is 0.001 mm. QA uses the existing case-specific deflection, including 0.00001 mm for the 0.01 mm wall. Runs are sequential; the second round reverses engine order. These are warmed medians from ten samples, not tail-latency guarantees.

[Metadata and binary hashes](data/metadata.json), [raw records](data/records.json), [summary](data/summary.json), [source hashes](sources.sha256), and [the exact dependency lockfile](benchmark-cargo.lock) preserve the inputs. The original parent binary was saved before the optimized build. The original `truck` checkout remained clean.

Reproduce with saved or rebuilt parent, optimized and OCCT runners, choosing a new output directory:

```bash
python3 truck-benchmarks/results/2026-10-10-analytic-holes/measure.py \
  --before target/analytic-holes/compare_before \
  --after target/analytic-holes/compare_after \
  --occt ../investigations/sequential-holes-2026-10-10/runner-build/compare_occt \
  --output target/analytic-holes-rerun
```

Measurements used the working tree in `truck-analytic-holes`, branch `optimize/analytic-hole-booleans`, before committing the changes.
