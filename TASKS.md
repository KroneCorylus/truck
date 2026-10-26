# Open work

The single list of open kernel work and how to split it across agents. Read
[AGENTS.md](AGENTS.md) first; its rules win over anything here. Updated 2026-10-25.

The main program is OCCT-level fillet/chamfer capability: dimensions, validity, editing
behavior and latency on equivalent modeling tasks, not matching OCCT's API or face structure.
Scope is the OCCT V7_9_3 inventory in
[audits/fillet-chamfer/coverage.json](audits/fillet-chamfer/coverage.json). Case keys below
are `native case/capture`; the capture is `001` unless shown.

## State

- Start every stream from current `master`.
- Known accepted test state: one ignored test, two existing Clippy warnings in shapeops
  tests, and the `pcurve_derivation` precision failure in truck-geometry.

Last full capture `5a4e3bc6`, 396 cases / 648 operations
([report](audits/fillet-chamfer/capture/2026-10-24-exact-knots/REPORT.md)), plus targeted
results since:

| Measure | Count |
| --- | ---: |
| Constructs | 483 |
| Limited comparison passes | 438 |
| Construction failures | 63 |
| Solid import failures | 7 |
| Inconsistent inputs | 6 |
| Comparison discrepancies | 45 |
| No-solid captures (scope review) | 88 + 8 cohort |

"Limited comparison" = valid output plus sampled two-way distance ≤ 0.001, not a global error
bound. The nine-case cohort ([report](audits/fillet-chamfer/capture/2026-10-24-nine-case-cohort/REPORT.md))
adds `blend/complex/D2` (construction failure) and 8 no-solid cases.

## Streams

### Completed batch — 2026-10-26

STEP malformed-knot handling, G2 quadratic remote fillets, C2 consumed chamfers, STEP
intersection export accuracy and G2 performance work are implemented and reviewed.
The changelog and committed tests/oracles describe the behavior; no agent remains active
from that batch. Original implementation baseline was `afedac31`.

Latest full existing-corpus replay: **486 chamfer operations, 357 constructions, 332 limited
comparison passes**, six C2 gains and no lost checks. Optimization retained every replay
status, diagnostic and sampled distance. A new full native recapture has not been run.
The broader optimization suite passed 893 tests with the documented pcurve_derivation
precision failure and two existing skips, plus 179 doctests and 77 replay tests. Final
review follow-ups added tests/docs only: 35 geometry unit tests, 5 integration tests and
139 doctests pass. Release Clippy retains the two known shapeops test warnings.

Matched G2 benchmark: **201.320 -> 25.463 ms**, fresh OCCT **29.703 ms**, 40 samples each
across alternating rounds on CPU0 with one worker. Repeated sampling/search grids and
first derivatives were the dominant costs. Approximately 217 KB more peak live cached
memory; no general never-slower guarantee. Neighboring benchmark medians showed no slowdown.
Independent review and derivative tests across weights, transforms, reversals and tiny/large
domains passed. Source/binary hashes and raw evidence are retained under:

- `target/g2-performance-20261026/RESULTS.txt` and `review-followups/validation.json`
- `target/g2-integration-validation-20261026/validation-results.json`
- `target/c2-panicfix-validation-20261026/comparison-details.json`
- `target/g2-matched-benchmark-20261026/RESULTS.txt`
- `target/integration-benchmarks-20261026/REPORT.md`

G2 reference accounting remains pending: native distances 1.13947 / 1.15467 exceed 0.001.
Independent sphere/material witnesses support Truck's rolling-ball result. Runtime tracing
shows OCCT chooses a three-boundary Coons fill, not a guaranteed constant-radius corner;
100x tighter approximation tolerance leaves the discrepancy unchanged. Treat this as a
mismatch with our rolling-ball contract, not a demonstrated OCCT API violation or intentional
speed shortcut. **Do not count it as a comparison pass.** Durable evidence:
[oracles/G2_MATERIAL.txt](audits/fillet-chamfer/oracles/G2_MATERIAL.txt),
`g2_corner_witness.cpp`, `g2_material.py`, and the matched benchmark harnesses.

### Next batch

Planned after committing the validated baseline: A2 curved chamfer corners, B1 G6/H1
fillet extension, and D remaining C1 chamfer sequences. Each gets a separate worktree;
release builds only with CARGO_BUILD_JOBS=8 and NEXTEST_TEST_THREADS=6. Timed benchmarks
must be serialized by the coordinator after implementation builds stop.

Each stream owns the files listed and can run in parallel with the others. A file marked
*shared* is edited by one stream at a time; ask the coordinator first. Notes under
[audits/fillet-chamfer/notes/](audits/fillet-chamfer/notes) are earlier agents' read-only
investigations: their measurements are leads, not proof, and none was run against current
`master`.

### A. Chamfer corners (22 construction failures, 3 discrepancies)

Owns `truck-shapeops/src/fillet/chamfer_miter.rs`, `chamfer_edges.rs`. Policy: follow OCCT's
corner construction (decided 2026-10-24). Reproduce OCCT behavior with
[probes/chamfer_corners.cpp](audits/fillet-chamfer/probes/chamfer_corners.cpp).

1. **Two-edge convex junctions.** OCCT intersects the two bevel half-spaces and adds no face;
   Truck adds a triangle transition when the bevel roots differ. New rule: tip =
   common face ∩ bevel a ∩ bevel b; Q = (shallower bevel's other face) ∩ bevel a ∩ bevel b;
   the deeper bevel runs to its root on the unselected edge. Concave junctions stay. OCCT
   volumes on box 10, edges 6 and 7: equal 1/1 → 990.333333333; 6(1) with 7(0.5) →
   993.854166667; two-distance variants → 996.354166667 and 995.104166667. First rewrite
   the tests that assert the convex triangle transition (start with
   `chamfer_transition_orientation.rs` and `chamfer_general.rs`). Target:
   `chamfer/{equal_dist,dist_dist,dist_angle}_complex/A1`, whose Truck→OCCT distance
   (0.053 / 0.038 / 0.014) remains after the three-edge corner rule (`2cf44b16`).
2. **Corners on curved supports and arc edges** (conical bevels):
   `chamfer/{equal_dist,dist_dist,dist_angle}_complex/{A4..A9,C2}` (21) and
   `chamfer/equal_dist_complex/C1`. Their final error `intersect_straight_blends` is the
   straight-edge fallback, not the first failure: record the first failing constructor per
   case before designing. Selections expand to 8–21 edges; partition them into local
   corners first. Lead: [triage §3](audits/fillet-chamfer/notes/triage-fillet-junctions-and-chamfers.txt).

### B. Fillet junctions on curved supports (22 cases)

Owns `truck-shapeops/src/fillet/planar_fillet.rs`, `planar.rs`.

1. **G6 (quartic), then H1 (cubic).** G2's bounded quadratic remote-junction construction
   is implemented; extend only with independently derived geometry and parent-failing
   regressions. See `tests/extrusion_remote_junction.rs` and the G2 oracle for the existing
   contract. Current guards intentionally exclude these higher-degree cases.
2. **F9, then G8, then G4**: two selected straight edges meeting at an extrusion corner with
   the curved cap edge unselected. Not the G2 topology and not a three-edge sphere. Exact
   source geometry and an oracle plan are in
   [notes/fillet-junctions-f9-g4-g8-i2.md](audits/fillet-chamfer/notes/fillet-junctions-f9-g4-g8-i2.md).
   Needs task T1 first: the replay currently reports the along-wire fallback error.
3. **I2** (quarter-revolved triangle, cone network; same note), then triage the rest:
   `blend/simple/{H3,H5,H9,I4,I8}`, `blend/simple/X1/002`, `blend/tolblend_simple/{D3,D5}`,
   `bugs/modalg_4/bug828`, `bugs/modalg_5/bug25657_1`, `bugs/modalg_7/bug30940_{2,4,5,6}`,
   `blend/complex/G9/002`.
4. F5 follow-up, only once a failing case exists: the extrusion branch samples blend
   orientation at chart point (0.5, 0.5), outside the fitted domain on short bodies.

### C. Unequal radii at a junction (6 cases)

Owns `truck-shapeops/src/fillet/planar_edge.rs`; `fillet/modeling.rs` is *shared* with B
(dispatch). Start with `blend/simple/P9`: box 5, radii 1 and 0.5 meeting at one vertex,
rejected today by the shared-vertex guard in `modeling.rs`. Reproduction, the existing test
expectation to rewrite, and a local analytic oracle are in
[notes/unequal-radii-p9.txt](audits/fillet-chamfer/notes/unequal-radii-p9.txt). Then
`blend/simple/A4`, `blend/tolblend_simple/A2`, `feat/featlf/{E2,E3}`, `blend/complex/D2`.
Earlier scope notes: `truck-benchmarks/results/2026-10-14-unequal-radii/REPORT.md`.

### D. Chamfer sequences (8 remaining cases)

Owns `truck-shapeops/src/fillet/along_wire.rs`.

1. `chamfer/{dist_dist,equal_dist}_sequence/C1` 002, 003.
2. `chamfer/dist_dist_sequence/C1` 007, 008 and `chamfer/equal_dist_sequence/C1` 006, 007:
   inputs are consistent; construction fails in `trim_faces`.

C2's six 005/010 targets are implemented; all 30 C2 captures pass. Preserve their analytic
regressions in `tests/d_c2_consumed_contour.rs`, the empty-contact rejection fix and
[material oracle](audits/fillet-chamfer/oracles/d-c2-contour-material-oracle.py).

### E. Imported inputs (13 cases)

Owns the STEP import / healing path (`truck-stepio`, import code in `truck-modeling`).
Design first (AGENTS.md: describe the design before a large change), since it needs an
explicit repair budget API.

- Import failures `chamfer/dist_angle_sequence/C1` 002–008: loose native vertices. Solve
  each shared vertex from its three unchanged supports and re-trim every incident curve.
- Inconsistent inputs `chamfer/dist_dist_sequence/C1` 004–006,
  `chamfer/equal_dist_sequence/C1` 004, 005, 008: edge interiors off their two supports by
  ~1.4–2.0e-6; the exact intersection lies slightly outside the bounded patch.

Fix by reconstructing geometry, never by loosening consistency tolerances. Measurements and
the proposed boundary are in [notes/imports-c1/](audits/fillet-chamfer/notes/imports-c1)
(the exact-knot part of `import-review.txt` has landed in `5a4e3bc6`).

### F. Discrepancies (45 cases, analysis first)

Read-mostly: produce per-case independent witnesses and route each fix to the owning stream.
Classification so far: [notes/discrepancies.txt](audits/fillet-chamfer/notes/discrepancies.txt)
(of the 43 rows reviewed, 6 show a demonstrated OCCT reference defect, 34 are unresolved and
3 are the A1 corners → stream A; H7 and F5 were added later).

- Blends: `blend/simple/{C2,C6,C8,D1,D9,E4,E6,E8,F3,F5,G1,G5,H4,H7,J6,J8,K2,K3,K4}`,
  `blend/tolblend_simple/A1`.
- Chamfers: `chamfer/dist_dist/{B3,B4,C1,C2,C3,C4,C6,C7,D1,D2,D5,D6,D9,E1}`,
  `chamfer/dist_angle/A2`, `chamfer/{dist_angle,dist_dist,equal_dist}_complex/A1`,
  `chamfer/{dist_angle,dist_dist,equal_dist}_sequence/C1/001`,
  `chamfer/dist_angle_sequence/{A6,A8}` 005 and 006.
- Start with the 14 `chamfer/dist_dist` cone-family rows: their repeated distances suggest one
  mechanism.
- A case where Truck matches an independent oracle and OCCT does not (F5: Truck within 6e-9,
  OCCT off by 2.5) is recorded as an exception with evidence, not counted as a pass. How such
  exceptions count is a user decision (G).

### G. Decisions for the user

- Which of the 96 no-solid captures (shell/open-surface tasks, including
  `blend/simple/{Q9,R1,R2,R3,R5,R6,U9,V7}`) are in scope
  ([notes/no-solid-scope.json](audits/fillet-chamfer/notes/no-solid-scope.json)).
- How a case counts when Truck matches an independent oracle but OCCT does not (stream F).
- App-side items from this audit (mixed-selection adapter, batch Remove booleans, debug
  `opt-level`, sketch fillets S01–S03) are in [../PiezaCAD/TASKS.md](../PiezaCAD/TASKS.md)
  stream B.

### H. Capability families

The scope inventory is [coverage.json](audits/fillet-chamfer/coverage.json) (OCCT V7_9_3,
51 families). Its status fields feed the generated catalog and are not maintained; this table
is the open list. Eleven families are implemented with tests (F01, C01, C02, G01, J01, J03,
N05, W03, W05, W07, W09). The rest, with the next concrete case to try:

| ID | Family | Status | Next case |
| --- | --- | --- | --- |
| F02 | Variable radius along a tangent chain | partial | Unequal edge lengths, split edges, same physical radius law, varying derivatives |
| F03 | Different radii meeting at a corner | unverified | Three edges at a corner with radii 1, 2, 3; convex then mixed (stream C) |
| F04 | Tapered radius reaching zero | partial | Radius sweeping to zero at an endpoint |
| F06 | Smoothness within and between blends | partial | Continuity in partially selected and asymmetric corners; two-edge fillet miters are creases |
| C03 | Distance-and-angle chamfers | unverified | Setback and angle on a non-right wedge, then after changing the wedge angle; curved support |
| C04 | Constant-throat chamfers | unverified | Recreate `bug28828_8`, `bug28828_10`; throat along the edge |
| C05 | Constant throat with penetration | unverified | Recreate `bug28828_9`, `bug28828_11`, NURBS `bug28828_6` |
| C06 | Meaning of distance on curved supports | behavior difference | Radius-2 cylinder, chord setback 1, each convention measured |
| G02 | Plane/cone and drafted rims | partial | Oblique plane/cone intersections, varied cone angles toward apex/contact collapse |
| G03 | Cylinder/cylinder, sphere, torus, spline pairs | partial | E9 torus trimming, G9 diagonal surface-curve import, non-coaxial cylinder/spline pairs |
| G04 | Analytic versus NURBS representations | partial | Non-cylindrical curved support pairs and split faces across analytic/NURBS/STEP |
| G05 | Curved and closed spline spines | partial | G1 discrepancy, F9/G2 (stream B), the six `bug30940` cases |
| T01 | Tangent propagation across support changes | partial | Open contours; several neighboring supports within one original edge |
| T02 | Closed chains and periodic seams | partial | Toroidal seams through terminations; closed single-edge spline contours |
| T03 | Branching or ambiguous chain selection | partial | A vertex with two tangent candidates; decide selection semantics |
| T04 | Open shells and free-boundary terminations | partial | Two-face open shell ending at a free boundary (scope decision G) |
| J02 | Single blend ending across several faces | partial | Nonplanar multi-face caps, moved seams |
| J04 | Two-edge junction, nonorthogonal supports | partial | Unequal draft angles and multi-blend configurations |
| J05 | Mixed convex/concave corners | partial | Each concavity pattern, selected subset and oblique angle; the opposite mixed-pair fillet |
| J06 | Four or more blends at a vertex | partial | Corners without a common spherical center; partially selected pyramid ridges |
| J07 | General curved multi-blend corners | partial | Cylinder/cone or spline supports at a multi-edge corner (streams A2, B) |
| D01 | Contact collapse and singular tips | partial | Non-tangent junctions, near-equal cylindrical corners |
| D02 | Disappearing support faces | partial | A thin face shrinking to zero under parallel and intersecting rounds |
| D03 | Disappearing unselected edges | partial | A chamfer crossing the exact limit where an unselected edge vanishes |
| D04 | Folded offsets and overlapping patches | partial | Radius growing through curvature and overlap limits: valid result or clear rejection |
| D05 | Contact crossing a trim loop or hole | partial | A hole or split boundary in the contact path, clearance across tolerance |
| D06 | Remote collisions and self-intersection | partial | A blend approaching a remote wall or cavity |
| N01 | Contacts near singularities | partial | Polar radius ratios, competing contact branches |
| N02 | Rapidly changing curvature | partial | Curvature spike between samples, long/short edge ratios, tight tolerance |
| N03 | Pcurve domains and seams | partial | G9 diagonal surface-curve import; ellipsoidal polar projection branches |
| N04 | Almost tangent, nearly coplanar edges | partial | Dihedral angle swept across each threshold at several scales |
| W01 | Sequential fillet/chamfer composition | partial | All four fillet/chamfer orders with nearby and intersecting selections (stream D) |
| W02 | Simultaneous straight and curved selections | unverified | Same mixed selection in varied orders (also the app adapter, G) |
| W04 | Stable references after topology edits | partial | Upstream edit after a face splits or disappears |
| W06 | Actionable failures and recovery | partial | One invalid selection inside a valid operation |
| W08 | Face-driven rolling ball, stopping faces | unverified | Face-pair fixture with an explicit stopping face |
| S01 | 2D line/arc fillets (app) | partial | Line-line, line-arc, arc-arc, both sides, oversized radii on trimmed segments |
| S02 | 2D general curves (app) | unverified | All `fillet2d` fixtures; underlying-curve versus trimmed-edge solutions |
| S03 | 2D chamfers (app) | unverified | Line-line sketch chamfers by setbacks or angle, then edit and remove |

F05 (fillet section representation) is reference only: act on it only after an observed
accuracy, STEP, modeling or performance failure.

### T. Tooling (coordinator)

1. `examples/replay_occt_blend.rs` (~line 1280) overwrites the multi-edge error from
   `try_fillet_solid_edges_with_radii` with the along-wire fallback's "not tangent
   continuous". Report both. Unblocks B2/B3.
2. The parked finite-facet mesher rewrite is on branch `f5-extrusion-candidate`
   (`6df3e1d7`). It changes existing replay meshes; revisit only as its own mesher change.
3. Original-scale F5 replay takes 11.4 s and 1.7 GB; it needs a benchmark median before any
   speed claim.

## Other kernel work

Checked against the code on 2026-10-25.

- **App tests broken by recent kernel commits.** Five cad-core failures in mesh volumes and
  STEP round trips have no known cause yet; bisect them here
  ([../PiezaCAD/TASKS.md](../PiezaCAD/TASKS.md) A1).
- **Offsets for shell and thicken.** Typed offsets of spline surfaces, curved thickening and
  general concave shelling (plane-plane concave joins already work). Blocks the app's
  surfacing (`../PiezaCAD/TASKS.md` C6).
- **Sweep corners on curved or closed paths.** Open line-only paths get bisector miters;
  curved and closed paths with corners still return `PathNotSmooth`.
- **Boolean performance** (through-holes already use the analytic path in
  `transversal/integrate/through_hole.rs`). Measure each with `truck-benchmarks` first:
  1. Exact `Line`/`Conic` cut edges for elementary surface pairs in general booleans, instead
     of `IntersectionCurve` Newton solves on every evaluation.
  2. Keep untouched faces and their meshes across sequential booleans;
     `triangulate_boundaries` re-meshes both operands on every call.
  3. `Edge::curve()` and `Face::surface()` clone the geometry on every access
     (`truck-topology`); add a borrowing accessor (breaking; check app call sites).
  4. `loops_store` vertex insertion walks every edge of every face; only the two faces sharing
     the edge matter.
  5. Unhinted `search_nearest_parameter(.., None, 100)` in `loops_store` and tangency checks;
     revolved elementary surfaces have no closed-form seed.
  6. HLR `blocked` inverts a 3×3 per triangle per curve piece; `interior_point`
     re-triangulates each undecided face.
  7. No Rayon in boolean intersection or face division; per-pair curves are independent.
  8. Ear clipping for planar faces with many holes: prototype gate is one 100-hole face
     from 6.7 ms to at most 1 ms, with a CDT fallback.

## Running several agents

One coordinator (the main session) owns `TASKS.md`, `CHANGELOG.md` conflicts, merges and full
captures. Every other agent runs one stream in its own worktree on its own branch from
`master` (T3 `delegate_task` with `isolation: "worktree"`, or `git worktree add`), and reports
results in its final message instead of editing this file.

- **Disk first.** Each worktree builds its own `target/` (about 3 GB release, 60 GB with
  debug builds), so build release only and delete worktrees when their branch merges.
  Timeshift snapshots `@home` daily and weekly, so deleted files free space only after the
  snapshots holding them rotate out or are removed.
- **Memory.** 31 GB RAM, 32 threads. Run at most three building agents at once, each with
  `CARGO_BUILD_JOBS=8` and `NEXTEST_TEST_THREADS=6`. Some replay tests peak near 2 GB.
- **Parallel-safe now:** A, B, C (C after B for `modeling.rs`), D, E, F. A starts with step 2;
  B with G6/H1; F needs no build slot for analysis.
- Each stream ends with a branch that passes the rules below on its own. The coordinator
  merges one branch at a time onto `master`, reruns the affected crates and
  `capture/replay_chamfers.py`, then updates the numbers and log here.

## Rules for each fix

1. Add a test that fails on the parent with identical fixtures. Use an independent oracle
   (analytic or flux volume), not OCCT output. Never loosen existing assertions.
2. Run `cargo nextest run --release -p <affected crates>`, their doctests, the
   `replay_occt_blend` example tests, all-target Clippy and the benchmark smoke. Benchmark
   measurements stay serial with `RAYON_NUM_THREADS=1`.
3. Replay the affected captures with the release replay binary and
   `target/occt-distance-tree-plugin/compare_shapes`.
   `audits/fillet-chamfer/capture/replay_chamfers.py OUT [GLOB]` does this for every chamfer
   capture in parallel and reports changes against the exact-knot capture (about 3 minutes).
4. Commit with a `CHANGELOG.md` line (only when the user has asked for commits).
5. Run a full capture at milestones, not after every fix. One worker, 120 s timeout (four
   workers ran out of memory). The output directory must not exist.

```bash
python3 audits/fillet-chamfer/capture/run_capture.py \
  --occt ../investigations/sequential-holes-2026-10-10/occt \
  --library-path target/occt-sdk/usr/lib64 --plugin target/occt-distance-tree-plugin \
  --replay target/release/examples/replay_occt_blend \
  --data-path target/occt-public-chamfer-data \
  --output target/occt-capture-<name> --workers 1 --timeout 120
```

`DRAWEXE` must be on PATH. The native OCCT checkout has three intentional local modifications.

**Keep when cleaning `target/`** (needed by the commands above; `cargo clean` deletes them):
`occt-sdk/`, `occt-distance-tree-plugin/`, `occt-public-chamfer-data/`,
`occt-capture-exact-knots-2026-10-24/`, `occt-capture-nine-case-cohort-2026-10-24/`.

## Log

| Date | Commit | Result |
| --- | --- | --- |
| 2026-10-22 | `9f97532a` | Torus corners; full run 480 construct / 437 pass |
| 2026-10-22 | `9e3ce08a` | Mixed curved corners; F1 passes, H7 constructs |
| 2026-10-24 | `5a4e3bc6` | Exact knots and STEP masters; full run 482 / 438 |
| 2026-10-24 | `834ee9f5` | Nine skipped native cases captured as a separate cohort |
| 2026-10-24 | `d3858852` | Elliptical extrusion corners; F5 constructs (discrepancy 0.107, OCCT reference suspect) |
| 2026-10-25 | `2cf44b16` | OCCT three-edge chamfer corners; chamfer replay 351 construct / 326 pass, unchanged |

Older trackers, handoffs and status reports were removed on 2026-10-25; they remain in git
history (`d86c6cfb` and earlier).
