# Seam-safe splitting of imported faces

The clean periodic-torus replay exposed a preprocessing panic on the second
operation of OCCT `blend/complex/G9`. The chosen divisor reused the endpoints
of an existing seam and created three outgoing edges at a junction. Divisor
selection now uses interior vertices of its two loops. Degenerate candidate
ranges return no divisor. Incomplete/unsupported boundary graphs return failure
instead of panicking; newly appended cut edges and polylines are discarded
when graph reconstruction fails.

The seam-selection unit regression checks every cyclic boundary start and
reversed traversal. It fails on the parent. The unchanged native G9 fixture
also reproduces the parent's panic. Its final regression exercises ordinary
and robust preprocessing before and after rigid motion. All four variants
require closed topology, a closed tessellation and independently calculated
volume using the shared solid harness. The fixture README derives that volume
from horizontal disk overlaps; it is approximately 6965.56812665.

The check concerns raw STEP geometry. G9 still fails conversion into modeling
geometry because a generated diagonal surface curve is unsupported. The final
targeted replay reports `input-import-failed` / `Unsupported imported geometry`,
without a panic. Modeling consistency, STEP re-export and the requested second
fillet are not established by this batch.

## Validation

- Full `truck-shapeops` release suite: **349 passed, 0 failed, 1 existing ignored**,
  68 test/doc targets. This includes the existing torus, tangent-boundary,
  spherical, spline and fillet regressions unchanged.
- All-target release Clippy for shapeops and the benchmark package: passed.
- Action benchmark smoke with one Rayon worker: passed.
- Final production replay build and targeted G9 replay: completed without panic.
- Touched Rust files formatted; existing assertions were not weakened or removed.

The exact commands, source/fixture/binary hashes and compiler are in metadata.json.
Parent failure logs and final validation logs are retained beside this report.
No performance comparison or speed claim is made for this batch.

The last complete native capture remains the clean 2099d2d8 run archived at
`audits/fillet-chamfer/capture/2026-10-12-periodic-tori/`. It predates this fix and
retains its recorded G9 panic, 80 constructions and 78 limited comparisons.
A new clean full capture is the first handoff task. Work stops at the user's
requested checkpoint; this batch does not establish full feature parity.
