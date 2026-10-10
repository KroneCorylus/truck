# Full benchmarks after analytic hole subtraction

All **72 Truck action workloads** completed on the parent and optimized kernels at **one and four workers**, with two rounds of 20 samples. All **20 direct OCCT cases** also completed at both worker counts: **240 preflight and timing records passed** geometry and volume validation. Truck is faster in **32 of the 40 case/worker comparisons** against OCCT 7.9.3.

The main remaining OCCT advantages are batch subtraction of multiple holes and filleting a bore rim. The two action measurements initially more than 10% slower than the parent did not reproduce in longer runs. Production code is unchanged from the preceding optimization.

## Direct OCCT comparison

One-worker medians in milliseconds. Each value is the median of two round medians, with 15 samples per round after warmup. A ratio above 1 favors Truck.

| Workload | Truck ms | OCCT ms | OCCT / Truck |
|---|---:|---:|---:|
| `holes_sequential/100` | 32.86 | 791.1 | 24.08× |
| `holes_batch/100` | 186.2 | 89.77 | 0.48× |
| `thin_wall/0.01` | 0.01744 | 1.533 | 87.91× |
| `fillet_bore/0.5` | 10.4 | 0.3096 | 0.03× |
| `thread/4` | 321.4 | 1547 | 4.81× |
| `knurl/128` | 86.59 | 307.3 | 3.55× |
| `mesh_plate/100` | 61.58 | 593.8 | 9.64× |

With four workers, 100 sequential holes take **32.85 ms** in Truck and **651.32 ms** in OCCT.

[Every OCCT result, tails, memory and validation status](occt/REPORT.md) · [Raw records](occt/records.jsonl) · [Machine, compiler, library and binary metadata](occt/metadata.json).

### Cases where OCCT remains faster

| Case | Workers | Truck ms | OCCT ms | Truck / OCCT |
|---|---:|---:|---:|---:|
| `holes_batch/10` | 1 | 14.55 | 9.33 | 1.56× |
| `holes_batch/10` | 4 | 11.92 | 6.824 | 1.75× |
| `holes_batch/30` | 1 | 44.76 | 26.57 | 1.68× |
| `holes_batch/30` | 4 | 37.32 | 18.97 | 1.97× |
| `holes_batch/100` | 1 | 186.2 | 89.77 | 2.07× |
| `holes_batch/100` | 4 | 162.2 | 61.91 | 2.62× |
| `fillet_bore/0.5` | 1 | 10.4 | 0.3096 | 33.61× |
| `fillet_bore/0.5` | 4 | 10.47 | 0.3179 | 32.94× |

These cases continue to use the general Boolean or fillet paths. The analytic through-hole optimization does not accelerate a compound of multiple cutters or the subsequent fillet operation.

## All Truck actions before and after

The action suite covers modeling, Booleans, local edits, STEP, tessellation and hidden-line projection. All 72 cases supplied all 20 requested samples in every run: **11,520 timed operations** across eight complete suites. Timed fallible operations and fixture sanity checks passed.

| Sequential holes | Workers | Parent ms | Optimized ms | Speedup |
|---:|---:|---:|---:|---:|
| 100 | 1 | 1224 | 33.49 | 36.55× |
| 100 | 4 | 813.5 | 33.61 | 24.20× |

The action fixtures generally use tolerance **0.01**, while the direct OCCT comparison uses **0.001**. The different parent timings across these suites are expected. Fixture construction and validation are outside operation timing.

[All 144 before/after action rows](actions/REPORT.md) · [Structured action results](actions/records.json) · [Action build and binary metadata](actions/metadata.json).

## Repeated measurements of apparent slowdowns

The full action run showed two four-worker rows more than 10% slower than the parent: `step/export_file/10` (+11.1%) and `tessellation/cylinder_tolerance/0.01` (+21.3%). Neither was over 10% slower in the follow-up. Both complete parameter families were rerun with **500 samples per case in each of five rounds**, alternating parent/optimized order, for 30,000 additional timed operations. All checks passed.

Times below are the median of five round medians. The original full-suite results remain in the action tables.

| Repeated case | Parent ms | Optimized ms | Change |
|---|---:|---:|---:|
| `step/export_file/1` | 0.08067 | 0.08428 | +4.48% |
| `step/export_file/10` | 0.2039 | 0.2041 | +0.10% |
| `step/export_file/30` | 0.4455 | 0.4512 | +1.28% |
| `tessellation/cylinder_tolerance/0.001` | 0.1782 | 0.1792 | +0.56% |
| `tessellation/cylinder_tolerance/0.01` | 0.05761 | 0.0566 | -1.75% |
| `tessellation/cylinder_tolerance/0.1` | 0.03873 | 0.0386 | -0.34% |

[Raw follow-up records](rechecks/records.json) · [Follow-up summary](rechecks/summary.json). These repetitions address the two flagged rows; small differences elsewhere remain subject to benchmark variation.

## Configuration and reproduction

- Date: 2026-10-10; AMD Ryzen 9 7950X3D; processes pinned to CPUs 0–3; Rayon/OCCT worker limits 1 and 4. Builds and timed processes ran sequentially.
- Parent: `3729a083932f5d73a29a4e84cc0b03866869aa27`. Optimized code: the `optimize/analytic-hole-booleans` working tree before committing the changes. [Source patch](occt/source.patch) and [source hashes](occt/sources.sha256) preserve the measured code; hashes were checked again after measurement. The parent checkout was unchanged throughout measurement.
- Full comparison uses the complete installed **OCCT 7.9.3** libraries with matching headers. The earlier focused hole investigation used a separate source build; use the within-run ratios here when comparing the full matrix. Linked libraries and their hashes are captured in metadata.
- OCCT comparisons use unchanged fixtures, fuzzy tolerance 0, the established case-specific QA deflection, and analytic volume error at most 0.2%. Both engines must pass and agree on volume before receiving a ratio. All 40 comparisons meet those conditions.
- These measurements cover the complete public action suite and direct OCCT matrix. They do not measure application rebuilds, GPU rendering, or the optional allocation-profiling configuration.

[Run orchestration](run.py), [follow-up orchestration](recheck.py), [execution records](execution.json), and raw logs preserve the commands. For another run, place `run.py` in a new sibling results directory before invoking it; it requires new output folders. The action command is:

```bash
RAYON_NUM_THREADS=1 taskset -c 0,1,2,3 /path/to/actions-binary \
  --bench --color never --sample-count 20 --sample-size 1 --max-time 60
```

The direct matrix command is recorded in `execution.json`. It uses `--samples 15 --rounds 2 --workers 1,4 --cpus 0,1,2,3` without a case filter.
