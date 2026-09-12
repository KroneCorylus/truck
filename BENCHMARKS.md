# CPU action benchmarks

The `truck-benchmarks` workspace package measures public CAD operations without changing
the production crates. It uses [Divan](https://docs.rs/divan/0.1.21/divan/) for repeated
measurements, compiler barriers, and optional allocation counts. Fixtures are generated
locally; no downloaded models, GPU, browser, or external files are needed.

## Running

```bash
# Run every workload once and check its result (release build).
RAYON_NUM_THREADS=1 cargo bench -p truck-benchmarks --bench actions -- --test

# Normal benchmark run: min/max/median/mean latency, samples and iterations.
RAYON_NUM_THREADS=1 cargo bench -p truck-benchmarks --bench actions

# Focus on one action; override the bounded defaults for a longer measurement.
RAYON_NUM_THREADS=1 cargo bench -p truck-benchmarks --bench actions -- subtract_sequential --sample-count 50 --max-time 60

# Separate run with allocation counting enabled.
RAYON_NUM_THREADS=1 cargo bench -p truck-benchmarks --bench actions --features allocations

# Discover individual cases and runner options.
cargo bench -p truck-benchmarks --bench actions -- --list
cargo bench -p truck-benchmarks --bench actions -- --help
```

Cargo uses its optimized bench profile. Fix `RAYON_NUM_THREADS` for comparisons; also run
with the thread count used by your application when assessing parallel tessellation.
Divan's `--threads` measures concurrent callers, not the Rayon worker count.

The boolean, blend, and projection cases request 20 samples and one operation per
sample, with a ten-second cap per boolean case and a two-second cap per blend or projection
case. A slow operation can overrun that cap; inspect the
reported sample count and increase `--max-time` before drawing conclusions from a few
samples. Other cases use Divan's adaptive sample size and default sample count.

## Workloads

| Group | Actions | Inputs |
| --- | --- | --- |
| `modeling` | Extrude, revolve | Profiles with 4, 32, 128 straight edges |
| `modeling` | Loft | 2, 8, 32 octagonal sections over the same height |
| `modeling` | Path sweep | Circular profile along 1, 8, 32 collinear path segments |
| `modeling` | Transform | Translate plates with 1, 10, 30 holes |
| `boolean` | Batch and sequential subtraction | Same plate/cutters, 1, 10, 30, 100 cylindrical holes |
| `boolean` | Union and intersection | Overlapping cylinders, chord tolerance 0.1, 0.01, 0.001; intersection also has a NURBS representation |
| `local` | Fillet and chamfer | 1 or all 12 edges of a cube |
| `local` | Draft, shell | 1 or 4 side faces; open-top box with 0.2 wall thickness |
| `tessellation` | Triangulate | Cylinder tolerance sweep; plates with 1, 10, 30, 100 holes |
| `tessellation` | Convert mesh to polygon | Already tessellated plates |
| `step` | Export, parse, convert, full import | In-memory STEP of plates with 1, 10, 30 holes |
| `projection` | Hidden-line projection | Isometric view of plates with 1, 10, 30 holes |

All plate cases use radius 1 holes on a pitch 4 grid and thickness 2. The grid expands with
hole count. General geometric tolerance is 0.01 in model units. The plate fixtures used
outside the boolean group are made by extruding a face with inner wires, so their setup
does not depend on boolean performance. The path-sweep workload measures segmentation
overhead on a straight path; it does not represent curved-path fitting.

Fixture construction and one preflight validation happen outside each timed closure.
Solid checks reject empty, open, partially tessellated, or non-positive-volume outputs;
plate and shell checks also compare an analytic volume. STEP checks require a solid,
the expected face count, and no skipped entities. Projection checks require visible and
hidden output. Timed fallible operations fail the run on errors. These checks are benchmark
sanity checks; the geometric regression suite remains the correctness authority.

Results are returned to Divan, which excludes their destruction from the timed operation.
Intermediate allocations and destruction inside an action are included. Inputs are reused
after preflight, measuring warmed operation latency rather than process startup or cold
caches. Transform measures geometry reconstruction, not a shallow `Solid::clone`.
STEP export includes topology compression and string generation. Full import includes
parsing and conversion to compressed STEP geometry; it excludes disk I/O, conversion into
the modeling enums, and tessellation. Parse and convert isolate those import stages.

## Choosing optimization work

Compare medians and scaling within an action before comparing unrelated actions. Normalize
by hole, edge, or section count when appropriate. A tenfold input increase taking much more
than ten times as long deserves investigation. Batch versus sequential subtraction is a
direct comparison of two ways to produce the same result. Tightening tolerance trades
accuracy for cost, so compare at the tolerance the application actually needs.

Measure allocations separately: instrumentation adds overhead, and Divan does not count
allocations on threads outside its control (including Rayon workers). The allocation
report is therefore a partial count for internally parallel actions, even with
`RAYON_NUM_THREADS=1`; it is not peak process memory. Do not compare instrumented timings
against an uninstrumented baseline.

For a boolean stage breakdown, use the plate diagnostic:

```bash
cargo test --release -p truck-shapeops --test bench_plate -- --ignored --nocapture
```

It reports triangulation, face pairing, interference, division, classification and fitting
for 10, 30 and 100 holes. These are single-run diagnostics, not statistical benchmarks.
The suite calls the `try_*` APIs, including their input checks; the plate diagnostic calls
the `Option` boolean API.

Keep the CPU, Rust version, commit, thread count, allocator feature, power mode, and
command with any saved result. Stop competing builds and heavy applications during
measurement. Repeat a suspected regression with the same settings before optimizing.
These are native CPU benchmarks; GPU rendering, WASM/JS overhead, assembly traversal,
and drafting sketch construction are outside this suite.

## Reference numbers

Medians measured at `91e6a0bc` on a Ryzen 9 7950X3D: Rust 1.98.1, system allocator,
powersave governor with boost, no affinity pinning, 20 to 30 samples per case. Use them to
spot a regression on the same machine, not as portable thresholds, and re-measure the parent
commit before comparing a change.

| Action, one Rayon worker | 30 holes | 100 holes |
| --- | ---: | ---: |
| Sequential subtraction | 116.7 ms | 1.269 s |
| Batch subtraction | 26.5 ms | 107.2 ms |
| Plate tessellation | 3.6–3.8 ms | 16.70 ms |
| Hidden-line projection | 11.9 ms | — |

Intersection of two cylinders represented by NURBS takes 5.04 / 14.72 / 207.2 ms at chord
tolerance 0.1 / 0.01 / 0.001. Full STEP import of the 30-hole plate took 7.55 ms at
`0af8c67a`, 7.36 ms of it parsing. No later change targeted STEP.

What these measurements established:

- Four Rayon workers are a good default for subtraction. Sequential subtraction at 30 holes
  drops from 116.7 ms with one worker to 77.8 ms with four. Eight workers add little and
  sixteen regress. Tessellation alone scales further, about 2.5 times with eight workers.
- Sequential subtraction rebuilds the growing result on every call. At 100 holes,
  triangulation and preparation take 999.7 of 1358.0 ms in the stage diagnostic, and batching
  disjoint cutters is about 12 times faster. Removing that cost needs reuse across operations,
  not more threads.
- STEP parsing dominates import and has not been optimized.
- Bulk CDT construction was tried and did not speed up sequential subtraction or plate
  tessellation.
