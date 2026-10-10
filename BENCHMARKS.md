# CPU action benchmarks

## Direct OpenCascade comparison

`truck-benchmarks/scripts/compare_occt.py` compares our native Rust kernel with a small
C++ runner linked directly to OCCT. It does not launch FreeCAD or include Python calls
inside an operation timer. The existing FreeCAD/document suite below remains useful for
application-level workflows.

On Linux, install CMake, a C++17 compiler, and OCCT development headers/libraries (7.8 or
newer, including `TKDESTEP`). Python uses only its standard library. The Rust example uses
the workspace's existing `serde_json` version for diagnostic records; no production crate
or production dependency changes.

From the Truck checkout:

```bash
python3 truck-benchmarks/scripts/compare_occt.py \
  --output truck-benchmarks/results/2026-10-10-occt \
  --samples 15 --rounds 2 --workers 1,4 --cpus 0,1,2,3

# Quick smoke check, or narrow a run with --filter thread.
python3 truck-benchmarks/scripts/compare_occt.py \
  --output target/occt-smoke-new --samples 1 --rounds 1 --workers 1 --cpus 0

python3 -m unittest discover -s truck-benchmarks/scripts -p test_compare_occt.py
```

Output directories must be new. Pick available physical CPUs on your machine; the defaults
are cores 0–3 in one L3 domain on this workstation. Every subprocess is pinned to that same
set. OCCT uses its native thread pool, limited to the requested worker count; its Boolean
and mesh parallelism is disabled with one worker. Truck uses `RAYON_NUM_THREADS`. These are
resource limits, not a claim of identical parallel algorithms.

If headers/libraries are outside the system search path, pass `--occt-include /path/to/include/opencascade`
and `--occt-lib /path/to/lib`. CMake stores those paths in `target/occt-comparison`.
On this Fedora workstation, OCCT 7.9.3 runtime libraries were already installed. Matching
`opencascade-devel-7.9.3-4.fc44.1.x86_64` headers were extracted under `target/occt-sdk`;
its development library symlinks point to the installed `/usr/lib64` runtime libraries.
The report's CMake cache records these local build paths. No system package changes are
required when using an existing matching SDK this way.

The 20 cases are:

| Case | Geometry and timed operation |
|---|---|
| `holes_batch/{1,10,30,100}` | Existing plate/cylindrical cutters, one compound subtraction |
| `holes_sequential/{1,10,30,100}` | Same material removal, one subtraction per hole |
| `knurl/{16,64,128}` | Radius 12, height 8 cylinder; disjoint triangular cutters form axial grooves of depth 0.35 |
| `thread/{1,4}` | External 60° thread on radius 4 shaft, pitch 1.25, depth 0.4, root flat pitch/8; one Boolean subtraction |
| `near_tangent/{0.01,0.001}` | Intersection of radius 1, height 2 cylinders with the specified radial overlap |
| `thin_wall/{0.1,0.01}` | Radius 4, height 8 cylinder bored through to the specified wall thickness |
| `fillet_bore/0.5` | Round the upper radius-3 through-hole rim in a 20 × 20 × 10 block |
| `mesh_plate/{1,100}` | Fresh triangulation and triangle extraction of the existing perforated plate |

All dimensions are millimetres. Knurl cutters clear the stock but stay inside disjoint
angular sectors; this is straight knurling, not a diamond knurl. Thread operands use Truck's
native rational spline cutter; the helper exports it to STEP and OCCT imports/checks it
outside the timer. Standard STEP transfer may heal geometry. The representations and face
counts can differ, so this is a matched workload rather than identical instruction streams.

Every case has an analytic expected volume. Thread volume integrates the groove's linear
60° flank over radius, for an integer number of turns; tools extend one pitch beyond each
end. Knurl volume sums each triangular notch plus its circular segment. Neither expected
volume is copied from either kernel's answer.

Each engine/case/worker configuration first runs an untimed preflight in its own process.
Only passing preflights proceed to timing. Each timed process reconstructs its fixtures,
performs one warmup, records the requested raw samples, then validates the last output.
Final-output destruction is outside the timer, while intermediate work is included. OCCT
booleans use non-destructive inputs and native tolerances with `--fuzzy 0` by default.
`--tolerance 0.001` controls Truck operations and both engines' absolute mesh deflection;
those settings are not equivalent geometric error guarantees. OCCT meshing uses a loose
π angular cap and copies the input topology without cached triangulations inside the timer.

Validation requires closed topology, native geometry consistency, complete closed oriented
meshes, positive volume and analytic volume error at most **0.2%**. Truck uses
`is_geometric_consistent`; OCCT uses `BRepCheck_Analyzer` with geometric/exact checks and
also integrates B-rep volume. Both merge coincident mesh positions at 1e-8 mm for closure
checks. The default QA mesh deflection is 0.001 mm, tightened to overlap/10000 for near
tangencies and wall/1000 for thin walls, with a shared 1e-6 mm floor (Truck's tessellation
minimum). CLI operation and QA tolerances must also be at least 1e-6 mm. This avoids judging small volumes using a coarse
validation mesh; it does not change the timed operation. Meshing cases validate the actual
timed mesh. These checks do not certify all surface deviations or equivalent topology.

Both engines must pass every round and agree on final mesh volume within 0.2% to earn a
ratio. Failures, panics, malformed output, missing samples and timeouts remain explicit;
failed preflights produce skipped timing rows. `--timeout` (default 180 seconds) bounds each
whole case process, including fixture setup and QA. A timeout therefore does not establish
an operation-latency lower bound. Timing records are flushed before QA so evidence survives
a subsequent validation timeout.

Builds and measurements run sequentially; every second timed round reverses execution
order. The report gives median of round medians, empirical P95, maximum and raw samples.
P95 from a small sample is descriptive, not a reliable tail guarantee. Peak RSS is read
before output QA, but includes process setup, libraries, warmup, thread pools and retained
outputs; thread setup also includes OCCT's STEP import/check. No aggregate speedup is computed.

The output preserves raw logs, `records.jsonl`, `metadata.json`, `summary.json`, `REPORT.md`,
input STEP files/hashes, source hashes, tracked and untracked source patches, `Cargo.lock`,
CMake settings, compiler versions and linked-library paths. Sources and binaries should
remain unchanged during a run. Rebuild a report without measurements using:

```bash
python3 truck-benchmarks/scripts/compare_occt.py \
  --output truck-benchmarks/results/2026-10-10-occt --report-only
```

OCCT reference contracts: [Boolean operations](https://occt3d.com/dev/doc/refman/html/class_b_rep_algo_a_p_i___cut.html),
[shape validation](https://occt3d.com/dev/doc/refman/html/class_b_rep_check___analyzer.html),
and [volume integration](https://occt3d.com/dev/doc/refman/html/class_b_rep_g_prop.html).

## Existing action suite

The [2026-09-20 fillet-search comparison](truck-benchmarks/results/2026-09-20/REPORT.md)
records the rolling-fillet optimization and its effect on the application's threaded box.

The [2026-09-12 comparison](truck-benchmarks/results/2026-09-12/REPORT.md) records PiezaCad
and Truck before/after measurements against installed FreeCAD 1.1.3, including remaining
losses and differences in the APIs being timed.

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
| `local` | Rolling fillet | One edge of a cube, including blend validation |
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

`step::export_file` adds file writing to export. `step::import_file` reads and converts a
shared STEP fixture, also read by FreeCAD. These fixtures are generated outside timing in
`target/comparison/shared-{1,10,30}.step`. Truck's import returns compressed topology;
FreeCAD additionally creates and heals its native B-rep. Keep that distinction in comparisons.

## Comparing with FreeCAD and the app

```bash
mkdir -p target/comparison
RAYON_NUM_THREADS=1 cargo bench -p truck-benchmarks --bench actions -- \
  --sample-count 20 --sample-size 1 --max-time 60 > target/comparison/truck-final.txt
RAYON_NUM_THREADS=1 cargo run --release --manifest-path ../PiezaCAD/Cargo.toml \
  -p cad-core --example bench_features > target/comparison/app-final.jsonl
timeout 300 flatpak run --command=FreeCADCmd org.freecad.FreeCAD \
  "$PWD/truck-benchmarks/scripts/compare_freecad.py" > target/comparison/freecad-final.txt 2>&1
```

The script also runs under another installation's `FreeCADCmd`; only the launch command
changes. Keep the scripts and data outside `/tmp` when using Flatpak. The FreeCAD log must
contain `COMPLETE` and no `FAILED`. Its `BENCH` lines are JSON; startup and console chatter
are outside timing. `CAD_BENCH_FILTER` optionally selects a FreeCAD action prefix (pass it
with Flatpak's `--env` option).

Run engines sequentially, after compilation finishes. Capture the original results as
`truck-before.txt` and `app-before.jsonl`, then run
`python3 truck-benchmarks/scripts/report_comparison.py target/comparison` to produce tables.
Repeat the final run and also test the production worker count. The app launcher defaults
to four workers, while the primary kernel comparison fixes one worker.

FreeCAD uses its native primitives and exact HLR. Its meshes are rebuilt from a copy without
cached triangulation using `MeshPart.meshFromShape`, absolute linear deflection and a loose
angular cap of π. Geometry and chord tolerance match the fixtures, but tessellation and
boolean tolerance policies are not identical. Loft interpolation, NURBS conversion and STEP
import have additional contract differences marked in the tables. Internal parsing,
conversion and mesh-merging stages have no matched FreeCAD public operation. The app's
corpus benchmark additionally covers sketches, constraints, targets, patterns, mirrors,
datums, feature references and supported failures, including full display-mesh construction.
These are workload comparisons, not a claim that every feature or input has equal coverage.

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
