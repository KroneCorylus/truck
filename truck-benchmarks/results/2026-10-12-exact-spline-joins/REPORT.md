# Exact spline joins preserve tight fits and STEP interoperability

The clean aa6cfa65 capture exposed 21 STEP read failures in OCCT. Fitted
cubic pieces retained duplicate endpoint controls and interior knots with
multiplicity four after removal of a tolerance-damaging optimization pass.
Truck's reader accepted that representation. The complete failed capture,
including every rejected output, is retained in
`audits/fillet-chamfer/capture/2026-10-12-conical-mesh-initial/`.

Spline concatenation now shares exactly equal endpoint controls and removes
their one redundant join knot. No approximation or global tolerance is used
for that reduction. Distinct nearby endpoints retain their original controls
and geometry. The tight fitter remains free of tolerance-based simplification.

Both regressions fail on aa6cfa65. The strengthened cubic-fit regression
requires legal interior multiplicities while retaining all position and
derivative checks, both scales, transformation and reversal. A second test
checks exact versus nearby joins, including preservation of positions and
derivatives on each side. No test was deleted, ignored or loosened.

The targeted original I3 replay now yields a closed mesh and an OCCT-valid
single-solid STEP. Its STEP volume is 979697.405982782 versus independent
979697.406914389. Bidirectional sampled distances are 0.0000113904573 and
0.0000084299785, at 1115 and 1106 stations. These are sampled checks, not
a certified global bound or full feature-parity verdict.

Full release geometry/modeling/meshalgo/shapeops/STEP: **868 passed**, two
pre-existing numerical property failures, five existing ignored tests,
135 targets. The previously recorded PCurve derivative failure remains.
The run also discovered a nearly coincident three-point circle-center
failure in the existing circular-arc test; the same saved seed fails on
parent aa6cfa65. Both parent evidence and the new circle seed are retained.
Clippy for all affected targets and the action smoke check pass.

## Final measurements against 713f0c9a

The same benchmark fixture and protocol as the conical-mesh report were
rerun sequentially after builds/tests finished: one warmup, five samples,
CPU 0 / CPUs 0-3 and one / four Rayon workers. The parent executable hash
was checked against the earlier baseline. Raw commands, samples and hashes
are in `measurements.json` and `metadata.json`.

| Action | Parent median, 1 / 4 workers | Fixed median, 1 / 4 workers |
| --- | --- | --- |
| Mesh, tolerance 10 | 467.493 / 270.766 ms, open | 0.043 / 0.105 ms, closed |
| Mesh, tolerance 0.001 | 30 s process timeout / timeout | 434.108 / 162.959 ms, closed |
| STEP, tolerance 0.00005 | 1.458 / 1.463 ms, succeeds | 1.450 / 1.433 ms, succeeds |
| STEP, scale 0.01, tolerance 0.0000005 | 2.477 / 2.470 ms, fails | 1.435 / 1.439 ms, succeeds |

No speedup ratio is assigned to failures or timeouts. The latter include
setup/warmup and are not operation-latency lower bounds. The prepared curves
now retain 98 controls, versus 128 in the initial fix and 64 in the older
original-scale export. The additional controls preserve tight fitting
accuracy; redundant exact duplicates are removed without changing geometry.

Public signatures and diagnostics are unchanged. This report validates the
targeted correction; a subsequent clean full capture checks recovery of
all previously accepted STEP exports and updates the audit totals.
