# Dense planar validation and cylindrical blend parity

The tube-shell median falls from **3.554 s to 160.4 ms** (22.2× faster, 95.5% less time)
with one Rayon worker. The app's unchanged tube workflow previously reached its 30-second
rebuild limit. After the validation change, its four shell cases completed in 321–381 ms
in an initial isolated run. Full-suite results below include the subsequent topology and
application naming corrections.

## Method

Baseline: Truck `7405a948`, PiezaCAD `48e8fe2`. The newly added `local::shell_tube` benchmark
was run before production edits, then again with the changes in this report. Its fixture
is an extruded annulus of height 10, outer radius 10 and bore radius 3, shelled inward by
0.5 with its top removed. Expected volume is `169π`, calculated by subtracting the offset
annular cavity. Fixture construction, mesh/volume preflight and selection are outside the
timer. The public operation is measured with Divan’s `bencher.bench(run)`. The benchmark
preflight uses mesh tolerance 0.01; independent kernel regressions use 0.001. The planar
validator still samples these shell boundaries at `TOLERANCE` (0.000001).

Both measurements used the same workstation and toolchain, `RAYON_NUM_THREADS=1`, five
samples with one iteration each, without simultaneous builds/tests or CPU affinity. This
is a local before/after comparison, not an OCCT timing comparison. The control operations
use the benchmark's existing sample counts. Raw logs, toolchain and source/executable
SHA-256 hashes are adjacent to this report. Benchmark source formatting changed between
runs; its fixture and timed work did not.

```sh
RAYON_NUM_THREADS=1 cargo bench --locked -p truck-benchmarks --bench actions -- shell_tube
RAYON_NUM_THREADS=1 cargo bench --locked -p truck-benchmarks --bench actions -- 'local::(fillet|chamfer|shell_box|rolling_fillet)'
```

| Operation | Before median | After median |
| --- | ---: | ---: |
| Shell annular tube | 3.554 s | 160.4 ms |
| Chamfer 1 box edge | 32.97 µs | 29.75 µs |
| Chamfer 12 box edges | 97.36 µs | 86.18 µs |
| Fillet 1 box edge | 96.76 µs | 85.34 µs |
| Fillet 12 box edges | 982.8 µs | 872.9 µs |
| Rolling fillet | 12.64 ms | 11.82 ms |
| Shell box | 108.0 µs | 112.1 µs |

Small control differences are not evidence of a general speedup. Box shelling is 3.8%
higher in the final run (4.1 µs); other controls are lower. These small differences do not
resolve significance relative to run-to-run noise. The final operation includes explicit face ancestry and component grouping.
The dense tube is the workload targeted by the optimization.

## What changed and what was checked

- Projected segment bounds discard distant pairs before the existing crossing/clearance
  predicates. Short loops retain exhaustive comparisons. Containment reuses the projected
  boundary and skips distance calculations outside padded segment bounds. Curve sampling,
  modeling tolerances, point/midpoint containment checks and the final predicates remain.
- Differential tests retain the old exhaustive validator and compare decisions for short
  and dense circles, concave/crossing/repeated-point loops, near contacts, rotated parts,
  and every qualifying pair in deterministic spatial fixtures. A 4,096-segment circle
  checks that fewer than four candidate pairs per segment reach the precise predicate.
- The new annular-solid check exposed an existing topology defect: two disconnected rims
  occupied one face. On the exact parent production sources the regression fails with
  genus 2 instead of 1 (72.92 s in the unoptimized test profile). Each opening boundary now
  creates its own face. Geometry and volume stay the same; this intentional topology
  correction changes face counts/order for multiply bounded openings. Removing a top around
  a blind pocket also separates its cup from the outer container. Connected boundaries are
  now grouped separately. `try_shell_with_history` records each outer/inner face's source;
  PiezaCAD uses those IDs instead of positional correspondence and resolves rims by their
  geometric hints.
- Annular and two-hole openings pass independent exact volumes, genus, closed tessellation,
  geometric consistency, unchanged-input and STEP round-trip checks, inward and outward,
  before and after a rigid transform. App references survive height edits, save/load and
  STEP export/import. Blind-pocket components retain their references across depth edits
  and are both recovered from the exported STEP file. The existing pocket regression was
  updated to require two physical components instead of one artificially connected shell;
  it retains the original total-volume assertion and additionally checks each component
  against its own exact volume and STEP round trip, in both shell directions.
- Circular rim fillets and unequal chamfers already constructed valid solids on analytic,
  exact NURBS and degree-elevated/shifted NURBS cylinders. Stronger round-trip checks exposed
  a STEP fitting failure on both NURBS cases. The exporter keeps its ordinary successful
  fits and, after those attempts fail, retries with source contact knots preserved. The
  same geometric tolerance is checked globally and within every fitted knot span. The
  three representations pass radius/normal or chamfer-section checks, analytic volume,
  closed meshes, immutable-input and STEP round-trip checks.

The prior app timeout evidence was captured in an isolated build at Truck `38aa7217` /
PiezaCAD `48e8fe2`; the relevant planar/shell sources were unchanged at `7405a948`.
`parent-app-timeout.txt` records the two 30-second timeouts. `parent-annular-regression.txt`
uses the exact `7405a948` production sources with the new test. `parent-exporter-regression.txt`
records the two NURBS failures with the unchanged parent STEP exporter. The application
correspondence failure after splitting rims, before migrating its adapter, is retained in
`parent-app-correspondence.txt`.

These are independent Truck/Pieza regression tasks, not executions of OCCT's 1,026 audited
case entries. General freeform support pairs, split-face equivalence and global collision
handling remain outside the demonstrated scope.

## Validation

- Full application suite: **427 passed, 0 failed, 2 existing ignored**. The unchanged
  tube and pocket workflows pass. The four shell integration tests were also rerun after
  the final export safeguard.
- Full shapeops suite: **299 passed, 1 existing failure, 1 existing ignored**. The failure
  is `r11_corners::unsupported_junctions_have_distinct_diagnostics`: a pyramid chamfer
  succeeds where the test expects `UnsupportedTopology`, as reproduced on earlier parents.
  That assertion remains unchanged.
- A subsequent review added a fourth curved-representation test: a longitudinal fillet
  domain scaled to `1e-9` panics in the parent knot evaluator during export. The exporter
  now returns `None` for domains below its evaluator resolution. Exact duplicate removal
  preserves distinct source knots, and the fallback retains the uniform fitter's maximum
  193 control points. This establishes safe failure, not arbitrary parameter-scale parity.
  All **15** targeted export/blend tests pass after this safeguard (including the new test),
  and the full STEP suite passes **79 tests**.
- Clippy passes for shapeops, stepio and benchmarks with all targets. Application Clippy
  completes with warnings in unchanged files. Targeted nightly rustfmt and diff checks pass.
  The action benchmark smoke suite and all eight audit-generator unit tests pass.

The full shapeops/app logs precede only the final export safeguard; the affected export
and shell checks were rerun afterward, as recorded in the separate logs. Final benchmark
measurements and source hashes include that safeguard. No existing assertion was weakened;
the pocket topology contract was corrected and strengthened as described above.
