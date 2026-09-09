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
sanity checks; the existing geometric regression suite remains the correctness authority.

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

For a boolean stage breakdown, the existing plate diagnostic remains available:

```bash
cargo test --release -p truck-shapeops --test bench_plate -- --ignored --nocapture
```

It reports triangulation, face pairing, interference, division, classification and fitting
for 10, 30 and 100 holes. These are single-run diagnostics, not statistical benchmarks.
The new suite uses the diagnostic `try_*` public APIs, including their input checks, while
that older diagnostic calls the legacy boolean API.

Keep the CPU, Rust version, commit, thread count, allocator feature, power mode, and
command with any saved result. Stop competing builds and heavy applications during
measurement. Repeat a suspected regression with the same settings before optimizing.
These are native CPU benchmarks; GPU rendering, WASM/JS overhead, assembly traversal,
and drafting sketch construction are outside this suite.

## Initial measurements (2026-09-08)

All 58 cases passed the release smoke run, two uninstrumented timing runs, and an allocation
run. The [recorded baseline](truck-benchmarks/baseline.txt) contains all 58 cases and machine
settings: Ryzen 9 7950X3D, Rust 1.98.1, one Rayon worker, system allocator, powersave governor
with boost enabled, no affinity pinning. Kernel revision was
`0af8c67aaa947d4ad23eda65bff8d1838fa2c02c` plus this benchmark suite. These are workstation
measurements, not portable performance thresholds.

The confirmation run used `--sample-count 30 --max-time 20`; all cases collected 30 samples.
The following are medians in milliseconds:

| Action | 1 hole | 10 holes | 30 holes |
| --- | ---: | ---: | ---: |
| Sequential subtraction | 4.37 | 64.78 | 357.90 |
| Batch subtraction | 4.16 | 40.46 | 120.90 |
| Hidden-line projection | 0.36 | 4.48 | 20.35 |
| Plate tessellation | 0.21 | 2.29 | 10.50 |
| Full STEP import | 0.77 | 2.91 | 7.55 |
| STEP parse only | 0.75 | 2.82 | 7.36 |
| STEP conversion only | 0.019 | 0.078 | 0.214 |

Suggested investigation order for these workloads:

1. **Repeated subtraction.** Going from 10 to 30 holes costs 5.5 times as much sequentially,
   compared with 3.0 times in a batch. Batching is about 3 times faster at 30 holes. Examine
   repeated tessellation/classification and intermediate-shape work using the stage diagnostic.
2. **Tessellation and hidden-line projection.** Both grow roughly 4.5–4.6 times from 10 to
   30 holes. Projection includes tessellation, so these costs overlap. Profile triangulation
   of multiply trimmed faces, then projection crossing and visibility work; the timings alone
   do not prove which internal algorithm is responsible.
3. **STEP parsing.** Parsing alone takes nearly as long as full import, whereas conversion
   takes 0.21 ms at 30 holes. Investigate parser allocation and entity handling first if import
   latency matters to the application.

The allocation run used `--features allocations -- --sample-count 10 --max-time 20` with
the same Rayon setting. At 30 holes, the median `alloc` count/bytes were about 424,336 /
156.2 MB for sequential subtraction, 109,842 / 23.89 MB for batch subtraction, and
180,825 / 8.384 MB for STEP parsing. These exclude reallocation growth and untracked worker
allocations; they measure allocation traffic, not retained or peak memory. They support
investigating allocation churn but do not establish it as the cause of the latency.

The largest modeling and local-edit cases stayed below 0.5 ms in this run. They are lower
priority for these fixtures; more complex curves, different geometry, or application call
frequency can change that ranking.

## Trimming optimization (2026-09-09)

Stage profiling confirmed that repeated tessellation consumed 57% of sequential subtraction
time at 30 holes and 86% at 100 holes. Triangle containment previously scanned every edge
of every trimming loop. Each loop now stores a parameter-space bounding box, so containment
skips loops that cannot contribute winding at the query point. The boxes are expanded by
the existing boundary tolerance to retain near-edge rejection.

Fresh measurements on the same workstation and kernel revision as above, using one Rayon
worker and 30 samples per case, produced these medians. The after column is a confirmation
run following the full 58-case benchmark and geometry regressions. See the
[raw before/after results](truck-benchmarks/optimization.txt) for commands, sample counts,
and stage timings.

| Action, 30 holes | Before | After | Time reduction |
| --- | ---: | ---: | ---: |
| Plate tessellation | 10.40 ms | 4.36 ms | 58% |
| Sequential subtraction | 355.70 ms | 258.20 ms | 27% |
| Hidden-line projection | 19.55 ms | 14.50 ms | 26% |

The 100-hole stage diagnostic improved from 5.40 s to 1.95 s for sequential subtraction
(2.8 times faster). Its tessellation stage fell from 4.62 s to 1.17 s. This larger case is
a single-run diagnostic, not a statistical latency estimate. Batch subtraction changed
little, as most of its time is in intersection and face division rather than tessellating
an already perforated plate. Batching remains faster when the operation permits it.

The optimization benefits faces with many separate trimming loops. Small workloads do not
show the same gains, and workstation timing variation remains visible. STEP parsing is
still a separate optimization candidate.

Validation: all 58 benchmark cases completed, and 334 regression/doc tests passed across
`truck-meshalgo`, `truck-shapeops`, `truck-hlr`, and `truck-stepio` (two pre-existing ignored
tests; the plate diagnostic was also run explicitly). Added checks cover holes, nested
islands, disconnected and concave regions, near-boundary points, and randomized equivalence
to the full winding scan. Release Clippy, formatting, and a WebAssembly tessellation build
check also passed.

## Native thread scaling (2026-09-09)

Measured the optimized kernel with 1, 2, 4, 8, and 16 Rayon workers, in ascending order and
then descending order to check for drift. Each configuration ran in a fresh process, with
20 samples of one operation per case. Runs were sequential, with no competing builds.
All 90 case runs completed their preflight checks and collected all 20 samples. The CPU,
governor, allocator, and build settings match the optimization measurements above.

The table shows the range of the two run medians, in milliseconds, for 30 holes. These
ranges are repeat measurements, not confidence intervals.

| Rayon workers | Sequential subtraction | Batch subtraction | Plate tessellation |
| ---: | ---: | ---: | ---: |
| 1 | 256.0–258.4 | 119.2–122.2 | 4.224–4.354 |
| 2 | 213.0–213.9 | 118.4–119.9 | 2.360–2.391 |
| 4 | 197.3–198.1 | 117.9–118.3 | 1.821–1.868 |
| 8 | 193.6–197.4 | 118.3–118.8 | 1.528–1.531 |
| 16 | 201.2–204.4 | 120.3–120.6 | 1.535–1.574 |

Existing parallel tessellation helps: four workers reduce sequential subtraction latency
by about 23% relative to one worker. Eight workers bring little additional improvement to
the full subtraction, and sixteen are slightly slower in both sweeps. Tessellation itself
improves about 2.8 times with eight workers, while batch subtraction changes little.

Four workers are a reasonable starting point for this native subtraction workload. The
results support focusing further work on repeated remeshing and the serial intersection
and face-division stages, rather than simply increasing the worker count. They do not
establish an ideal thread count for other models or for multiple concurrent CAD jobs.
The current WebAssembly tessellation path remains single-threaded.

Reproduce an individual configuration with:

```bash
RAYON_NUM_THREADS=4 cargo bench -p truck-benchmarks --bench actions -- 'subtract_(batch|sequential)|tessellation::plate' --sample-count 20 --sample-size 1 --max-time 120
```

The [complete thread-scaling output](truck-benchmarks/thread-scaling.txt) includes all
1-, 10-, and 30-hole cases from both sweeps. Only measurements and documentation were added;
the kernel and its default thread settings were not changed by this comparison.

## Cylinder search and intersection optimization (2026-09-09)

This pass compares against the already optimized kernel from the trimming and thread
studies above. A copy of its bench executable was retained before making changes, then
run alongside the final executable on the same workstation. Each comparison used 30
samples of one operation, with two runs per configuration and reversed execution order.
No build or other benchmark ran concurrently. Tolerance, fixtures, allocator, compiler,
and machine settings match the earlier measurements.

The table gives the range of the two medians, in milliseconds, for 30 holes. Repeated
medians are not confidence intervals.

| Action | Workers | Before this pass | After this pass | Approximate speedup |
| --- | ---: | ---: | ---: | ---: |
| Sequential subtraction | 1 | 253.6–257.7 | 116.7–116.7 | 2.2× |
| Batch subtraction | 1 | 119.4–119.6 | 26.44–26.50 | 4.5× |
| Sequential subtraction | 4 | 196.8–197.0 | 77.84–77.87 | 2.5× |
| Batch subtraction | 4 | 116.0–116.7 | 24.22–24.62 | 4.8× |
| Plate tessellation | 1 | 4.173–4.201 | 3.563–3.762 | 1.1–1.2× |
| Hidden-line projection | 1 | 13.60–14.14 | 11.94–11.98 | 1.1–1.2× |

CPU sampling initially attributed about 43% of cycles to matrix-vector multiplication
and 9% to sine/cosine evaluation. Circular-cylinder surface searches repeatedly evaluated
51 sampled rulings to obtain a starting parameter. The retained changes are:

- A circular-cylinder inverse supplies the starting parameters for nearest-point searches.
  The existing Newton solver still refines the result. Exact parameter searches can return
  the inverse directly only after the original surface evaluation passes the existing
  point-residual tolerance. Angular hints retain their nearby periodic branch. Restricted
  search ranges, noncircular or skew extrusions, degenerate inputs, and failed candidates
  retain the general search.
- Intersection projection evaluates only position and the two first derivatives it needs,
  avoiding two fixed 121-entry derivative tables per Newton iteration.
- Face intersection work materializes each polygon mesh and boundary polyline once per
  pass. Padded boundary boxes reject distant edges before the original contact checks.
- Mesh refinement reuses vertex positions across adjacent facets within each iteration.

A separate comparison changed only the derivative evaluation. For intersection of the same
cylinders represented by NURBS, medians at tolerances 0.1 / 0.01 / 0.001 changed from
5.404 / 16.11 / 210.5 ms to 5.042 / 14.72 / 207.2 ms (one worker, 50 samples each).
The tightest case shows little benefit, but no regression appeared in these measurements.
Bulk CDT construction was also tried and removed because it did not improve sequential
subtraction or perforated-plate tessellation.

The updated worker sweep produced these median ranges for 30 holes:

| Workers | Sequential subtraction | Batch subtraction | Plate tessellation |
| ---: | ---: | ---: | ---: |
| 1 | 116.70–116.70 | 26.44–26.50 | 3.563–3.762 |
| 2 | 87.51–88.85 | 25.17–28.23 | 1.959–2.028 |
| 4 | 77.84–77.87 | 24.22–24.62 | 1.590–1.659 |
| 8 | 75.75–76.18 | 24.68–25.23 | 1.449–1.509 |
| 16 | 84.09–85.42 | 26.98–27.10 | 1.575–1.677 |

Four workers remain a useful starting point. Eight save only about 2 ms on the full
sequential operation; sixteen regress. No global thread-pool setting was changed.
The code also compiles for WebAssembly, but these latency measurements are native only.

The suite now contains 64 cases, including 100-hole subtraction/tessellation and the
NURBS-cylinder intersection sweep. See the [complete measurements](truck-benchmarks/deep-optimization.txt)
for commands, both comparison runs, the worker sweep, the full suite, and stage timings.

The 20-sample full-suite run measured the 100-hole case at **1.269 s sequentially**,
**107.2 ms in a batch**, and **16.70 ms for plate tessellation** on one worker.
Batching the disjoint cutters is nearly 12 times faster than sequential calls after these
optimizations. The stage diagnostic still assigns 71.2 of 121.7 ms at 30 holes and
999.7 of 1358.0 ms at 100 holes to triangulation/preparation (one worker). These stage
numbers are separate single runs. Sequential calls rebuild the growing result repeatedly;
removing that scaling cost would require reuse across operations or combining compatible
cuts. More worker threads alone do not remove it.

Validation of the final source:

- All 64 benchmark cases completed all 20 samples and their preflight checks.
- The release suite across geometry, modeling, shape operations, meshing, hidden-line
  projection, and STEP completed with **638 passed, 2 failed, and 5 pre-existing ignored**.
  Both failures are existing `truck-geometry/tests/circle.rs` inverse-parameter precision
  assertions. The identical inputs and numeric errors reproduce in a test executable from
  September 5. The general circle implementation and its tests remain unchanged. A global
  `atan2` replacement was rejected because it disturbed an imported STEP mesh; all STEP
  tests pass with the retained changes. An existing randomized B-spline cut test also
  failed once earlier, then passed its rerun and the final suite.
- Added tests cover transformed/reversed cylinders, shifted and partial angular ranges,
  radial offsets, angular hints and periodic branches, unsupported/degenerate fallback,
  and tolerance-sensitive boundary-box rejection. Existing geometric regressions cover
  Boolean topology and volume, intersections, blends, and imported models.
- The 10/30/100-hole topology/volume diagnostic passed with both one and four workers.
  Release Clippy, formatting, and WebAssembly build checks passed.

The [validation log](truck-benchmarks/deep-validation.txt) records the commands, full final
suite output, matching pre-existing failures, and build checks. The two existing precision
failures mean the full suite is **not** reported as green.
