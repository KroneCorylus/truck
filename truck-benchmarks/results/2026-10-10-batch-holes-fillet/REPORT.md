# Analytic batch holes and faster fillet validation

With one worker, 100 batch holes fall from **186.65 ms to 0.920 ms**: **203× faster than
the parent** and **97× faster than OCCT 7.9.3** on this fixture. Bore-rim filleting falls
from **10.41 ms to 1.85 ms**, a **5.62× improvement**. OCCT still wins the bore fillet at
0.315 ms, about 5.88× faster than the optimized Truck result.

All **20 direct cases** and **72 action workloads** were measured before and after with
one and four workers. All benchmark checks passed. Truck leads OCCT in **38 of 40 direct
case/worker comparisons**; bore-rim filleting remains the only losing case at both worker
counts. These are fixture-specific results, not a claim about arbitrary CAD models.

## Main results

Times are milliseconds, reported as the median of two round medians with 15 timed samples
per round after warmup. Parent and optimized binaries were measured in the same run.

| Operation | Workers | Parent ms | Optimized ms | OCCT ms | Parent / optimized |
|---|---:|---:|---:|---:|---:|
| 100 batch holes | 1 | 186.648 | 0.920 | 89.538 | 202.95× |
| 100 batch holes | 4 | 160.371 | 0.885 | 62.103 | 181.19× |
| Bore-rim fillet, radius 0.5 | 1 | 10.409 | 1.851 | 0.315 | 5.62× |
| Bore-rim fillet, radius 0.5 | 4 | 10.526 | 1.887 | 0.336 | 5.58× |
| 100 sequential holes | 1 | 33.250 | 33.636 | 794.948 | 0.99× |
| 100 sequential holes | 4 | 33.263 | 33.465 | 648.503 | 0.99× |

The 10- and 30-hole batches also improve: one-worker medians are 0.108 and 0.300 ms,
respectively, versus 14.355 and 44.940 ms on the parent. The existing sequential-hole
optimization remains close to its parent timing. No direct case measured more than 10%
slower than the parent.

[All 40 direct comparisons and round medians](data/direct/summary.json) ·
[All 360 preflight/timing records, raw samples and QA results](data/direct/records.jsonl).

## Implementation

Batch subtraction now recognizes a compound of disjoint, parallel circular through-hole
cutters and applies the existing analytic construction to every cutter. It collects the
openings and rebuilds each affected cap once. Unaffected faces and existing bore walls
retain their identities; input solids remain unchanged. This avoids the general Boolean
pipeline's meshing, intersection, boundary insertion and curve fitting for accepted batches.

The fast path conservatively checks complete cutter projections for separation, including
the small axis tilt admitted by the existing angular tolerance. Overlapping, touching,
nested, inward, blind, noncircular or otherwise unsupported members defer the whole batch
to the general solver. Multiple target shells remain unsupported by the analytic path.
Cutters entirely in empty space remain valid no-ops.

Sampled curve inclusion now predicts the next surface parameters from the previous two
accepted points. Each predicted search must return finite parameters within the surface
range and reproduce the point within the existing geometric tolerance. A bad prediction,
failed solve or invalid result falls back to the original unseeded search. The same 33
curve samples and iteration limit remain in place.

This removes much of the repeated global parameter search during fillet validation.
It does not replace Truck's general fillet surface with a special analytic torus, so the
remaining bore-fillet gap is not eliminated. There are no public API changes, new
dependencies or new unsafe code.

## Correctness and regression tests

All **360 direct records passed** native geometry checks, closed oriented mesh checks and
the analytic volume threshold of **0.2%**. All **40 cross-engine comparisons** also agree
on mesh volume within 0.2%. The default operation and QA mesh tolerances are 0.001 mm;
near-tangent and thin-wall cases retain the established tighter QA deflections. These
checks do not certify all possible surface deviations.

The batch fast path changes the curve representation and resulting tessellation. For
100 holes, Truck retains 206 faces but uses exact circular boundaries rather than the
general solver's fitted boundaries. At the same QA deflection:

| 100-hole result | Faces | Triangles | Analytic volume error |
|---|---:|---:|---:|
| Parent Truck | 206 | 51,612 | 0.009810% |
| Optimized Truck | 206 | 29,212 | 0.030999% |
| OCCT | 106 | 40,412 | 0.016073% |

All errors remain below 0.2%. Different representations produce different meshes; fewer
triangles do not by themselves establish equivalent accuracy. Bore-fillet geometry is
unchanged in this benchmark: both Truck versions produce 10 faces, 8,708 triangles and
the same measured volume, with 0.002429% analytic volume error.

Seven new tests cover transformed/reordered batches, differing radii and arc segmentation,
opposite cutter axes, exact curves, preserved face identity, unchanged inputs, STEP round
trips, no-op cutters, existing holes, mixed through/blind fallback, ambiguous cutter
rejection, sampled-search reuse, invalid seeded results, off-surface curves, poles, seams
and transformed surfaces.

Two representative regression tests were run against the unchanged parent implementation
and failed as intended: the search-reuse test observed 33 global searches, and the batch
test observed zero preserved remote faces instead of four. Both pass with this change.
See [parent search regression](validation/parent-inclusion.log) and
[parent batch regression](validation/parent-batch.log).

The full release test run for `truck-modeling` and `truck-shapeops` reports **361 passed,
1 failed and 3 already ignored**. The sole failure is
`r11_corners::unsupported_junctions_have_distinct_diagnostics`, whose chamfer assertion
expects an error but receives a solid. It also fails on unchanged parent `ad3b76fa`;
the test was not changed or ignored. See [full test log](validation/tests.log) and
[parent reproduction](validation/parent-diagnostic.log).

Release Clippy for both crates and all targets passed. The complete action smoke run
passed, and changed Rust files pass the workspace formatter.

## Complete action suite

All 72 actions completed in each of eight suite runs: parent and optimized, one and four
workers, two rounds of 20 samples. That is **11,520 timed operations**; timed fallible
operations and fixture checks passed. The action suite generally uses tolerance 0.01,
while the direct OCCT suite uses 0.001, so the parent batch timings differ.

| Action | Workers | Parent ms | Optimized ms | Parent / optimized |
|---|---:|---:|---:|---:|
| 100 batch holes | 1 | 66.410 | 0.913 | 72.74× |
| 100 batch holes | 4 | 64.285 | 0.900 | 71.45× |
| Rolling fillet | 1 | 16.190 | 12.795 | 1.27× |
| Rolling fillet | 4 | 16.365 | 12.470 | 1.31× |
| 12-edge fillet | 1 | 0.866 | 0.907 | 0.95× |
| 12-edge fillet | 4 | 0.856 | 0.891 | 0.96× |

The changes do not improve every workload. The 12-edge fillet measured about 4–5% slower.
Small differences remain subject to run variation; the complete results are retained in
[all 144 action comparisons](data/actions/summary.json) and
[action execution records](data/actions/records.json).

### Follow-up on the apparent tessellation slowdown

The initial four-worker `tessellation/cylinder_tolerance/0.001` result was 0.22555 ms
before and 0.25275 ms after, a 12.1% slowdown. The full cylinder-tolerance family was
repeated with 500 samples per case in five rounds, alternating execution order. All
15,000 additional timed operations passed their checks.

| Cylinder mesh tolerance | Parent ms | Optimized ms | Change |
|---|---:|---:|---:|
| 0.001 | 0.18340 | 0.18960 | +3.38% |
| 0.01 | 0.05727 | 0.05775 | +0.84% |
| 0.1 | 0.03968 | 0.03920 | −1.21% |

The slowdown above 10% did not reproduce. This follow-up does not prove statistical
equivalence. [Follow-up summary](data/rechecks/summary.json) ·
[Follow-up commands and records](data/rechecks/records.json).

## Measurement and reproduction

- Date: 2026-10-10. Same workstation as the preceding full run; Linux x86-64,
  Rust 1.98.1, OCCT 7.9.3. All measured processes used CPU affinity 0–3 and worker
  limits of one or four. Builds and measured processes ran sequentially.
- Parent: `ad3b76fa393d7db89d274866cf955acc1a8cce4f`. Saved parent binaries were checked
  against the preceding full run's binary hashes; relevant parent source hashes and the
  unchanged dependency lockfile were also checked. [Baseline provenance](baseline.json).
- Optimized code: the uncommitted `optimize/batch-holes-fillet-validation` working tree,
  captured in the [source patch](data/source.patch) and [source hashes](data/sources.sha256).
  Source and binary hashes were verified unchanged after measurement.
- Direct runs use an untimed preflight and two timed rounds per engine, case and worker
  count: 360 process records and 3,600 timed operations. Each timed process warms up
  first. The second round reverses engine order. Fixture construction and output QA are
  outside operation timers. OCCT fuzzy tolerance is zero. Thread operands are exported
  to STEP outside timing; fixture hashes are recorded.
- Ratios compare median operation times within this run. Neither an aggregate speedup
  nor an application rebuild speedup is inferred. GPU paths and optional allocation
  profiling are outside this suite.

[Compiler, library paths, settings and binary hashes](data/metadata.json) ·
[Dependency lockfile](data/Cargo.lock) · [Measurement runner](measure.py) ·
[Follow-up runner](recheck.py) · [Measurement completion log](validation/measurement.log).

To repeat with separately built parent and optimized release binaries, use a new output
directory (the saved result directory must not be overwritten):

```bash
python3 -B truck-benchmarks/results/2026-10-10-batch-holes-fillet/measure.py \
  --before /path/to/parent-compare \
  --after /path/to/optimized-compare \
  --occt /path/to/compare_occt \
  --actions-before /path/to/parent-actions \
  --actions-after /path/to/optimized-actions \
  --output target/batch-fillet-repeat
```

The compare executable is built with
`cargo build --release --locked -p truck-benchmarks --example compare_occt`; the action
executable with `cargo bench --locked -p truck-benchmarks --bench actions --no-run`.
Copy each built executable before switching revisions. Use the same dependency lockfile
and OCCT build for both Truck revisions. Direct OCCT fixture and timing contracts are
documented in [BENCHMARKS.md](../../../BENCHMARKS.md).
