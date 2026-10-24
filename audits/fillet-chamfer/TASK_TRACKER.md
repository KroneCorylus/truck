# Fillet and chamfer completion tracker and agent handoff

Updated: **2026-10-24**. Owner: Krone. Repository: `/home/krone/Development/Pieza_WS/truck`, branch `master`. Verified implementation HEAD: **`9e3ce08a`**. This handoff covers **all remaining work**, including the complete captured-operation backlog and the broader capability audit, not only the next checkpoint.

The objective is OCCT-level fillet/chamfer capability and robustness for equivalent modeling tasks in Truck/Pieza. Judge physical dimensions, geometry, validity, editing behavior, and practical latency. Matching OCCT APIs, internal algorithms, patch types, or face counts is not the objective. The scope is the pinned OCCT V7_9_3 inventory in [coverage.json](coverage.json), not every possible mathematical configuration or future OCCT release.

Implementation resumed on 2026-10-24 at Krone’s request with multiple subagents; the full mixed-corner capture is complete and verified; implementation validation is next. This document prepares work for another agent; creating it does not start kernel work or launch subagents. The receiving agent should follow Krone's new assignment and stop conditions. Previous authorization covered achievement commits; no push, PR, consumer dependency change, or unrelated application change is authorized by this handoff.

## Start here

1. Read [repository instructions](../../AGENTS.md), this tracker, and the [latest detailed handoff](checkpoints/2026-10-22-mixed-curved-implementation/HANDOFF.txt).
2. Inspect the actual branch, HEAD, working tree, local artifacts, and active processes. Do not assume this snapshot remains current. Preserve other people's changes.
3. Assign a coordinator and work packages below. Record agent names, branches, and base commits in the assignment table before starting.
4. First complete package A's fresh full corpus capture on the committed mixed-corner implementation. Keep the root source and replay binary frozen during capture. Other agents can inspect evidence and prepare isolated worktrees in parallel.
5. Integrate validated achievements one at a time; update the operation checklist, capability checklist, checkpoint evidence, and activity log at each checkpoint.

**Two separate completion levels:** the captured corpus is a finite regression baseline; the capability audit also includes scenarios, product behavior, and sketch operations outside that corpus. Finishing the 122 baseline unresolved operations does not establish full parity.

## Verified state and completed achievements

- [x] Toroidal corner implementation committed as `066b4880`.
- [x] Complete torus corpus capture and report committed as `9f97532a`.
- [x] Mixed F1/H7 implementation, tests, proof, report, and restart handoff committed as `9e3ce08a`.
- [x] Mixed regression evidence: 12 procedural operations, 24 imported operations, four new oversized rejections, and preserved six existing oversized torus rejections.
- [x] Identical-source parent failures demonstrated for new procedural/imported tests and the intentionally expanded F1 contract.
- [x] Mixed validation: 471 shape-operation tests, 119 STEP tests, 76 replay tests, 13 capture tests, all-target Clippy, single-thread benchmark smoke, and native reference/volume/distance checks (8/27/24).
- [x] Final production F1/H7 original-scale replays construct consistent closed STEP with unchanged inputs. F1 passes the limited comparison; H7 still fails its distance threshold.
- [x] Full mixed-corner corpus capture and comparison against the previous complete run; archive verified (8,497 members). Catalog refresh tracked below.

The latest complete run is the **2026-10-24 mixed-corner checkpoint**, with 396 native cases and 648 captured operations. All 480 previously successful result/comparison records are exactly preserved:

| Measure | Confirmed count |
| --- | ---: |
| Operations that construct | 482 |
| Operations passing the limited comparison | 438 |
| Positive solid reference operations | 559 |
| Unresolved positive solid operations | 121 |
| Construction failures within those 121 | 60 |
| Solid import failures | 7 |
| Inconsistent solid inputs | 10 |
| Constructed comparison discrepancies | 44 |
| Other captures with no input solid | 88 |
| Matched native rejection | 1 |

“Constructs” means the operation produced geometry; it does not establish shape equivalence. Limited comparison includes the recorded validity/material-processing checks and sampled bidirectional distance threshold of 0.001; it is not a global geometric error bound. The 88 no-solid inputs require scope review for shell/sketch tasks and must not be silently discarded from the broader audit.

F1's targeted distances are approximately 0.0000399792 / 0.0000399774. H7's are 0.001525464 / 0.001514833. The complete run now confirms **482 constructions and 438 limited passes**. F1 is the sole new limited pass; H7 moves from construction failure to discrepancy. All 648 operation keys/tasks, native references and 2,590 native geometry files (excluding only STEP capture timestamps) are preserved. The historical 122-row ledger below retains original dispositions and now has one resolved row.

One existing ignored test, two existing test warnings, and the known `pcurve_derivation` precision failure remain explicit. Dependency validation was reused only with matching source hashes. Do not describe the whole workspace as passing all tests.

## Subagent assignment and integration plan

Use separate worktrees for concurrent implementation. These packages are independent investigation boundaries, not a promise of disjoint production files. B, C, E, and F can overlap in `truck-shapeops/src/fillet`; D and G can overlap in geometry, meshing, and STEP. The coordinator resolves ownership and integrates focused commits. Never copy whole experimental directories over production.

At most **three workers plus one coordinator** can run with the currently available four slots. Rotate packages in waves. Suggested first wave: B investigates F5 tessellation, D investigates input reconstruction, G independently reviews discrepancies, while A runs the frozen baseline capture. Next waves assign C/E/F and then H/I. H and I may start read-only product investigations earlier if a worker slot is free.

**Shared machine limits:** one Cargo build/test/bench at a time, `CARGO_BUILD_JOBS=12`; one full capture at a time with `--workers 1 --timeout 120`. Separate worktrees do not eliminate memory pressure or shared Cargo cache hazards. Coordinate build slots; use isolated target directories where necessary and never rebuild the production replay during capture. Read-only analysis and independent small calculations can proceed concurrently.

| Package | Scope | Agent and worktree | State | Integration dependency |
| --- | --- | --- | --- | --- |
| A | Coordinator, corpus, catalog, integration | `/root`, root `master` @ `9e3ce08a` | Mixed capture/catalog checkpoint complete; integration ongoing | First capture precedes new root kernel changes |
| B | Extruded curved corners and F5 tessellation | `/root/b_f5`, root `master` @ `9e3ce08a`; scoped scratch patch | Candidate proven in scratch; formal parent/full validation pending | Coordinate remote corner mechanism with C |
| C | Remaining curved junctions and collapse | Unassigned | Open | Integrate after A; share geometry changes with B/F |
| D | STEP imports, pcurves, seams, inconsistent inputs | `/root/d_import`, root `master` @ `9e3ce08a`; scoped scratch patch | Exact-knot candidate prepared; support gaps remain | Coordinate mesher/STEP changes with G |
| E | Chamfer construction and dimensional modes | Unassigned | Open | D owns input repair; G owns discrepancy analysis |
| F | Unequal, variable, and zero-radius fillets | Unassigned | Open | Coordinate common corner construction with C |
| G | Independent geometry proof, discrepancies, robustness, performance | `/root/g_proof`, read-only root `master` @ `9e3ce08a` | Discrepancy/scope reviews complete; candidate reviews recorded | Supplies oracles and reviews to all packages |
| H | Downstream modeling, references, diagnostics, application workflows | `/root/g_proof`, sibling PiezaCAD @ `f5a381704` | Source review complete; runtime/app work pending | Read-only app review until app changes authorized |
| I | Sketch fillets and chamfers | `/root/g_proof`, sibling PiezaCAD @ `f5a381704` | Source/scope review complete; implementation authorization pending | Confirm product edit scope before application changes |

### Common assignment prompt

Copy this paragraph with the chosen package section when delegating:

> Read `audits/fillet-chamfer/TASK_TRACKER.md` and `AGENTS.md`. Work only on package [ID] from base commit [SHA], in [WORKTREE]. Coordinate shared Cargo/capture resources with [COORDINATOR]. Review the associated operation and capability checklist entries; claim each exact case/capture before changing it. Preserve existing assertions, canonical comparison thresholds, physical selections, and input immutability. New capability/fix tests must fail on the parent with identical fixtures. Return focused commits when authorized, reproduction commands, test/benchmark evidence, before/after case results, unresolved limitations, and the next action. Do not update shared corpus/catalog files, push, change consumer pins, or launch further agents unless assigned. Stop at [SPECIFIC BOUNDARY].

### Package A coordinator and complete corpus accounting

Own this tracker, `audits/fillet-chamfer/capture`, shared catalog/coverage updates, integration commits, and checkpoint reports. Delegate only claimed cases and scoped files. The coordinator is the only writer of shared progress totals.

- [x] Confirm source/replay hashes against `truck-benchmarks/results/2026-10-22-mixed-curved-corners/validation.json`; rebuild if needed without changing source.
- [x] Confirm DRAWEXE, SDK, plugin, public data, native checkout snapshot, and a fresh output path. Rechecked 444 source hashes and replay SHA256; native checkout has three tracked modifications plus existing untracked `src/BOPAlgo/HoleProfile.hxx`, preserved.
- [x] Run the complete mixed capture and preserve runtime logs and every artifact hash.
- [x] Check all 480 prior successful Truck and comparison records for exact preservation. Investigate every change; never rewrite the baseline to hide a regression.
- [x] Reconcile all 648 operation keys, native tasks, physical selections, input scopes, tolerance records, and source/executable hashes.
- [x] Update this ledger from actual results, retaining original state and links to new proof for resolved rows.
- [x] Archive capture evidence, verify archive members/hashes, refresh catalog, run catalog/capture tests, and check desktop/mobile HTML plus evidence links. 8,497 archive members, catalog `--check`, audit tests and 13 capture tests pass; report checked at 1100/390 px.
- [x] Commit the capture/report achievement separately from implementation (2026-10-24 mixed capture checkpoint).
- [ ] Review native inventory omissions and skipped/missing fixtures; acquire reproducible public fixtures with provenance or record the specific remaining blocker. Missing fixtures are not passes.
- [ ] Coordinate all remaining packages and perform the final completion review below.

Capture command, after prerequisites and root cleanliness are confirmed:

```bash
cd /home/krone/Development/Pieza_WS/truck
python3 audits/fillet-chamfer/capture/run_capture.py \
  --occt ../investigations/sequential-holes-2026-10-10/occt \
  --library-path target/occt-sdk/usr/lib64 \
  --plugin target/occt-distance-tree-plugin \
  --replay target/release/examples/replay_occt_blend \
  --data-path target/occt-public-chamfer-data \
  --output target/occt-capture-mixed-curved-corners \
  --workers 1 --timeout 120
```

`DRAWEXE` must be available on PATH. The output directory must not already exist. Four-worker capture previously caused a confirmed out-of-memory failure. Preserve the native checkout's three recorded preexisting modifications.

Prepared local helpers: `target/package-mixed-curved-corners-full.py` and `target/write-mixed-curved-corner-checkpoint.py`. Read before running: they assume particular paths/counts, are not generally idempotent, and the writer's continuation/F5 narrative predates the stop request. Update those assumptions from evidence. If old raw capture directories are missing, recover their archived artifacts and verify hashes. Do not rerun `integrate-mixed-curved-corners.py`.

### Package B extruded corners and F5 performance

Primary production candidates are `fillet/planar.rs` and `fillet/planar_fillet.rs`. The isolated prototype is `target/extrusion-corner-source`; preserve current mixed `along_wire.rs`/`modeling.rs` when porting. F5 has independent remote ends; G2/G6/H1 require a two-selected-edge remote junction, shared with C.

- [ ] Preserve/review the authoritative prototype and its independent volume derivation.
- [ ] Locate why F5 fitted face 5 tessellates to 505,675 points / 1,008,827 triangles. Inspect boundary UV ranges, periodic branches, parameter division, projection outside the fitted domain, and refinement before choosing a fix.
- [ ] Fix the demonstrated cause without reducing geometric precision; establish meaningful before/after benchmark medians.
- [ ] Port procedural/native tests with repository fixtures; prove parent failures and check formatting/Clippy.
- [ ] Integrate the focused F5 source changes, run complete affected validation, rebuild final production replay, and recheck original-scale F5.
- [ ] Commit implementation/proof; coordinator captures and archives the full corpus afterward.
- [ ] Resolve G2/G6/H1 remote canal junctions with C; center solving alone is not successful construction.

Existing evidence: 12 refined procedural operations plus four oversized rejections pass; two imported F5 order controls pass. Material uses an independent boundary-flux oracle. The original-scale prototype constructs but retains sampled distances about 0.1069274 / 0.1056517. It is not production proof.

Timing diagnosis: construction about 48 ms, radius/tangency complete at 82 ms, raw material checks finish at 76.604 s, STEP at 76.800 s. A separate per-face trace attributes 27.824 s to face 5; the other faces take under 8 ms each. **Raw tessellation, not STEP, is the unresolved cost.** These traces locate work; they are not formal benchmark medians.

Local sources: `target/extrusion_corner_junctions.rs`, `target/extrusion-native-module.rs`, `target/extrusion-final-procedural-tests.txt`, `target/extrusion-final-native-tests.txt`, `target/extrusion-corner-volume.cpp`, `target/extrusion-corner-volume-method.txt`, `target/generic-extrusion-corner-design.txt`, `target/extrusion-corner-timing.rs/.txt`, and `target/extrusion-corner-mesh-timing.rs/.txt`.

Do not rerun the prepare/refine prototype scripts blindly: later periodic-branch fixes are absent from the initial generator. `extrusion-corner-planar-clean.rs` is stale. The refined library and procedural/native test binaries differ from the pre-refinement `extrusion-corner-replay`; rebuild before final claims. The initial wrong ellipse branch and later failed representation assumption are diagnostic history, not reasons to relax validity tests.

### Package C curved junctions and topology transitions

Own general fillet junction construction outside B/F's claimed cases. Start from the exact operation rows, not abbreviated case names alone.

- [ ] Diagnose remaining F9/G4/G8/I2 and bug-regression construction failures before proposing a shared mechanism.
- [ ] Resolve H3 remote polar termination; H5 curved common support; H9 outside-neighbor cone contact; I4 conical and I8 freeform junctions.
- [ ] Resolve X1's radius-15 request around a preexisting radius-10 convex cylinder using a valid corner construction.
- [ ] Work with B on G2/G6/H1 remote two-edge junctions.
- [ ] Extend mixed concavity, partial selections, oblique ends, four-plus-edge corners, and topology collapse according to J/D family outcomes below.
- [ ] Validate contact dimensions, continuity, independent material, trimming, closed output, STEP, selection order, and rejected sizes; obtain parent proof and final original-scale replays.

### Package D imports and parameter geometry

Own input reconstruction and STEP/pcurve/seam mechanisms; coordinate mesher changes with G.

- [ ] Resolve seven `chamfer/dist_angle_sequence/C1` solid-import failures.
- [ ] Resolve ten inconsistent C1 inputs in distance-distance/equal-distance sequences by reconstructing endpoint geometry; do not loosen consistency tolerances.
- [ ] Resolve G9's different-station seam/contact configuration in cooperation with C. Its current captured status is construction failure, so do not assume the historical import failure still describes it.
- [ ] Validate analytic/NURBS equivalence, split faces, moved seams, inversions, parameter scaling, and STEP round trips for affected mechanisms.
- [ ] Review no-solid captures for legitimate shell/sketch tasks with H/I; distinguish import loss from intended non-solid geometry.

### Package E chamfers

Own construction failures on valid chamfer inputs and C-family dimensional semantics. D owns failed imports; G owns initial discrepancy diagnosis; return geometry fixes to E after diagnosis.

- [ ] Resolve the complex equal-distance, distance-distance, and distance-angle construction failures listed in the ledger.
- [ ] Resolve sequential C2 requests where a 0.5 chamfer consumes a previous 0.2 chamfer, plus remaining valid-input C1 sequence failures.
- [ ] Validate side identity and physical setback conventions under selection/storage reversal and curved supports.
- [ ] Complete distance-angle, constant-throat, and penetrating-throat outcomes, trying equivalent existing constructions before introducing APIs.
- [ ] Validate intended dimensions after upstream angle/curvature edits and retain canonical discrepancy records until resolved with evidence.

### Package F radius laws and unequal junctions

Own different radii, physical radius laws, and zero-radius endpoints. Coordinate shared corner filling with C.

- [ ] Resolve captured A4/P9, tolerance-grid A2, and featlf E2/E3 unequal-radius requests; verify the exact intended profile for each fixture.
- [ ] Validate physical radius profiles on unequal-length and split chains, including derivative variation and reversed selection.
- [ ] Establish zero-radius endpoint behavior from actual native executions and independent geometry; support valid tapering without invalid seams.
- [ ] Prove joined convex/mixed-concavity corners with distinct incident radii, plus oversized/inconsistent requests and immutable failure.

### Package G independent proof and numerical robustness

Own initial investigation of all 43 baseline constructed discrepancies and newly constructed discrepancies such as H7/F5. Supply independent oracles to implementing packages rather than assuming native output is infallible.

- [x] Initial review of all 43 baseline discrepancies plus H7 is recorded in [the discrepancy review](checkpoints/2026-10-24-mixed-curved-corners/g-discrepancy-review-2026-10-24.txt); unresolved cases remain unresolved. Classify every discrepancy: physical intent/convention, native reference defect, Truck construction defect, STEP fitting, comparator defect, or unresolved. Preserve raw canonical results in every category.
- [ ] Validate H7 and F5 analytic witness diagnoses against final production output. Two worst-point witnesses do not establish a global error bound.
- [ ] Preserve and review prior C1 guide, C2/C8 sphere, E4 volume, G1, and tolerance-grid A1 diagnoses without silently changing thresholds/statuses.
- [ ] Resolve general self-intersection/material-membership checks, distant collisions, crossed trim loops, folded offsets, and disappearing support/contact cases.
- [ ] Sweep near-tangent/coplanar thresholds, high curvature, scale/translation, small retained features, singular contacts, and competing branches.
- [ ] Measure latency with `truck-benchmarks` before claiming speedups; review F5 performance with B and output-quality changes with D.

Do not restart the stopped E4 original-scale STEP stress automatically or call it a pass. The separate standard original-scale ellipsoid regression passed. Review existing evidence first. A reference diagnosis can explain a discrepancy but does not make the canonical comparison pass; record any accepted reference exception separately with evidence and user agreement.

### Package H modeling workflows and delivery

Own W-family coverage, shell scope, propagation/selection behavior, diagnostics, and final product-readiness evidence. Inspect the actual consumer location: repository instructions mention `test_cad`, while historical audit artifacts refer to `../PiezaCAD`; establish which checkout/pin is authoritative before application work.

- [ ] Verify mixed straight/curved selections, Boolean-generated intersections, all relevant feature orders, and stable physical selection after upstream edits.
- [ ] Verify face history after splits/deletions/replacements and actionable failure/retry without input mutation.
- [ ] Validate downstream Booleans, meshing, STEP interchange, face-driven rolling-ball/stopping-face outcomes, and open-shell/free-boundary tasks.
- [ ] Identify required consumer changes and prepare a scoped implementation/pin migration proposal. Do not change the app or its dependency pin without explicit authorization.
- [ ] Coordinate end-to-end product validation after kernel achievements and authorized integration.

### Package I sketch operations

Own S01/S02/S03. This extends beyond the solid replay corpus and requires inspecting the actual application sketch implementation.

- [ ] Verify line-line, line-circle, and circle-circle fillets on both sides and trimmed domains.
- [ ] Verify ellipse/spline corners, extension, multiple solutions, and selection of the intended solution.
- [ ] Establish editable sketch chamfers with setbacks/angles, modification/removal, connectivity, and constraints.
- [ ] Record reproducible missing outcomes and propose application changes; implement only within explicitly authorized app scope.

## Complete operation checklist

These **122 rows are generated from the last complete torus manifest** and are keyed by full native case path plus capture number. They are the full unresolved positive-solid baseline, not a shortlist. Family IDs such as `F01` below are separate identifiers from native case names such as `blend/simple/F1`.

A checked operation requires a recorded final disposition and evidence: a confirmed complete-run pass, or an explicitly agreed reference/scope exception kept separate from pass totals. A prototype or successful construction alone is insufficient. Add `agent / commit / evidence / remaining issue` to each claimed row. After each complete run, reconcile all operation keys and add any newly exposed regressions; do not delete unresolved rows.

Initial package assignment is a triage owner, not a proven root cause. G transfers construction fixes to B/C/D/E/F once diagnosed. The full mixed capture resolves F1; H7 remains unchecked as a constructed discrepancy. Baseline rows are retained for traceability.

### Construction failures

- [ ] `blend/complex/G9` / `002` — package **D**.
- [ ] `blend/simple/A4` / `001` — package **F**.
- [x] `blend/simple/F1` / `001` — package **A**; full 2026-10-24 capture confirms limited pass at `9e3ce08a`; [proof](capture/2026-10-24-mixed-curved-corners/comparison-summary.json).
- [ ] `blend/simple/F5` / `001` — package **B**.
- [ ] `blend/simple/F9` / `001` — package **C**.
- [ ] `blend/simple/G2` / `001` — package **B**.
- [ ] `blend/simple/G4` / `001` — package **C**.
- [ ] `blend/simple/G6` / `001` — package **B**.
- [ ] `blend/simple/G8` / `001` — package **C**.
- [ ] `blend/simple/H1` / `001` — package **B**.
- [ ] `blend/simple/H3` / `001` — package **C**.
- [ ] `blend/simple/H5` / `001` — package **C**.
- [ ] `blend/simple/H7` / `001` — package **G**; full 2026-10-24 capture confirms construction, but comparison still fails. Retained here as original construction-failure ledger entry; current category is discrepancy.
- [ ] `blend/simple/H9` / `001` — package **C**.
- [ ] `blend/simple/I2` / `001` — package **C**.
- [ ] `blend/simple/I4` / `001` — package **C**.
- [ ] `blend/simple/I8` / `001` — package **C**.
- [ ] `blend/simple/P9` / `001` — package **F**.
- [ ] `blend/simple/X1` / `002` — package **C**.
- [ ] `blend/tolblend_simple/A2` / `001` — package **F**.
- [ ] `blend/tolblend_simple/D3` / `001` — package **C**.
- [ ] `blend/tolblend_simple/D5` / `001` — package **C**.
- [ ] `bugs/modalg_4/bug828` / `001` — package **C**.
- [ ] `bugs/modalg_5/bug25657_1` / `001` — package **C**.
- [ ] `bugs/modalg_7/bug30940_2` / `001` — package **C**.
- [ ] `bugs/modalg_7/bug30940_4` / `001` — package **C**.
- [ ] `bugs/modalg_7/bug30940_5` / `001` — package **C**.
- [ ] `bugs/modalg_7/bug30940_6` / `001` — package **C**.
- [ ] `chamfer/dist_angle_complex/A4` / `001` — package **E**.
- [ ] `chamfer/dist_angle_complex/A5` / `001` — package **E**.
- [ ] `chamfer/dist_angle_complex/A6` / `001` — package **E**.
- [ ] `chamfer/dist_angle_complex/A7` / `001` — package **E**.
- [ ] `chamfer/dist_angle_complex/A8` / `001` — package **E**.
- [ ] `chamfer/dist_angle_complex/A9` / `001` — package **E**.
- [ ] `chamfer/dist_angle_complex/C2` / `001` — package **E**.
- [ ] `chamfer/dist_angle_sequence/C2` / `005` — package **E**.
- [ ] `chamfer/dist_angle_sequence/C2` / `010` — package **E**.
- [ ] `chamfer/dist_dist_complex/A4` / `001` — package **E**.
- [ ] `chamfer/dist_dist_complex/A5` / `001` — package **E**.
- [ ] `chamfer/dist_dist_complex/A6` / `001` — package **E**.
- [ ] `chamfer/dist_dist_complex/A7` / `001` — package **E**.
- [ ] `chamfer/dist_dist_complex/A8` / `001` — package **E**.
- [ ] `chamfer/dist_dist_complex/A9` / `001` — package **E**.
- [ ] `chamfer/dist_dist_complex/C2` / `001` — package **E**.
- [ ] `chamfer/dist_dist_sequence/C1` / `002` — package **E**.
- [ ] `chamfer/dist_dist_sequence/C1` / `003` — package **E**.
- [ ] `chamfer/dist_dist_sequence/C2` / `005` — package **E**.
- [ ] `chamfer/dist_dist_sequence/C2` / `010` — package **E**.
- [ ] `chamfer/equal_dist_complex/A4` / `001` — package **E**.
- [ ] `chamfer/equal_dist_complex/A5` / `001` — package **E**.
- [ ] `chamfer/equal_dist_complex/A6` / `001` — package **E**.
- [ ] `chamfer/equal_dist_complex/A7` / `001` — package **E**.
- [ ] `chamfer/equal_dist_complex/A8` / `001` — package **E**.
- [ ] `chamfer/equal_dist_complex/A9` / `001` — package **E**.
- [ ] `chamfer/equal_dist_complex/C1` / `001` — package **E**.
- [ ] `chamfer/equal_dist_complex/C2` / `001` — package **E**.
- [ ] `chamfer/equal_dist_sequence/C1` / `002` — package **E**.
- [ ] `chamfer/equal_dist_sequence/C1` / `003` — package **E**.
- [ ] `chamfer/equal_dist_sequence/C2` / `005` — package **E**.
- [ ] `chamfer/equal_dist_sequence/C2` / `010` — package **E**.
- [ ] `feat/featlf/E2` / `001` — package **F**.
- [ ] `feat/featlf/E3` / `001` — package **F**.

### Solid import failures

- [ ] `chamfer/dist_angle_sequence/C1` / `002` — package **D**.
- [ ] `chamfer/dist_angle_sequence/C1` / `003` — package **D**.
- [ ] `chamfer/dist_angle_sequence/C1` / `004` — package **D**.
- [ ] `chamfer/dist_angle_sequence/C1` / `005` — package **D**.
- [ ] `chamfer/dist_angle_sequence/C1` / `006` — package **D**.
- [ ] `chamfer/dist_angle_sequence/C1` / `007` — package **D**.
- [ ] `chamfer/dist_angle_sequence/C1` / `008` — package **D**.

### Inconsistent inputs

- [ ] `chamfer/dist_dist_sequence/C1` / `004` — package **D**.
- [ ] `chamfer/dist_dist_sequence/C1` / `005` — package **D**.
- [ ] `chamfer/dist_dist_sequence/C1` / `006` — package **D**.
- [ ] `chamfer/dist_dist_sequence/C1` / `007` — package **D**.
- [ ] `chamfer/dist_dist_sequence/C1` / `008` — package **D**.
- [ ] `chamfer/equal_dist_sequence/C1` / `004` — package **D**.
- [ ] `chamfer/equal_dist_sequence/C1` / `005` — package **D**.
- [ ] `chamfer/equal_dist_sequence/C1` / `006` — package **D**.
- [ ] `chamfer/equal_dist_sequence/C1` / `007` — package **D**.
- [ ] `chamfer/equal_dist_sequence/C1` / `008` — package **D**.

### Constructed comparison discrepancies

- [ ] `blend/simple/C2` / `001` — package **G**.
- [ ] `blend/simple/C6` / `001` — package **G**.
- [ ] `blend/simple/C8` / `001` — package **G**.
- [ ] `blend/simple/D1` / `001` — package **G**.
- [ ] `blend/simple/D9` / `001` — package **G**.
- [ ] `blend/simple/E4` / `001` — package **G**.
- [ ] `blend/simple/E6` / `001` — package **G**.
- [ ] `blend/simple/E8` / `001` — package **G**.
- [ ] `blend/simple/F3` / `001` — package **G**.
- [ ] `blend/simple/G1` / `001` — package **G**.
- [ ] `blend/simple/G5` / `001` — package **G**.
- [ ] `blend/simple/H4` / `001` — package **G**.
- [ ] `blend/simple/J6` / `001` — package **G**.
- [ ] `blend/simple/J8` / `001` — package **G**.
- [ ] `blend/simple/K2` / `001` — package **G**.
- [ ] `blend/simple/K3` / `001` — package **G**.
- [ ] `blend/simple/K4` / `001` — package **G**.
- [ ] `blend/tolblend_simple/A1` / `001` — package **G**.
- [ ] `chamfer/dist_angle/A2` / `001` — package **G**.
- [ ] `chamfer/dist_angle_complex/A1` / `001` — package **G**.
- [ ] `chamfer/dist_angle_sequence/A6` / `005` — package **G**.
- [ ] `chamfer/dist_angle_sequence/A6` / `006` — package **G**.
- [ ] `chamfer/dist_angle_sequence/A8` / `005` — package **G**.
- [ ] `chamfer/dist_angle_sequence/A8` / `006` — package **G**.
- [ ] `chamfer/dist_angle_sequence/C1` / `001` — package **G**.
- [ ] `chamfer/dist_dist/B3` / `001` — package **G**.
- [ ] `chamfer/dist_dist/B4` / `001` — package **G**.
- [ ] `chamfer/dist_dist/C1` / `001` — package **G**.
- [ ] `chamfer/dist_dist/C2` / `001` — package **G**.
- [ ] `chamfer/dist_dist/C3` / `001` — package **G**.
- [ ] `chamfer/dist_dist/C4` / `001` — package **G**.
- [ ] `chamfer/dist_dist/C6` / `001` — package **G**.
- [ ] `chamfer/dist_dist/C7` / `001` — package **G**.
- [ ] `chamfer/dist_dist/D1` / `001` — package **G**.
- [ ] `chamfer/dist_dist/D2` / `001` — package **G**.
- [ ] `chamfer/dist_dist/D5` / `001` — package **G**.
- [ ] `chamfer/dist_dist/D6` / `001` — package **G**.
- [ ] `chamfer/dist_dist/D9` / `001` — package **G**.
- [ ] `chamfer/dist_dist/E1` / `001` — package **G**.
- [ ] `chamfer/dist_dist_complex/A1` / `001` — package **G**.
- [ ] `chamfer/dist_dist_sequence/C1` / `001` — package **G**.
- [ ] `chamfer/equal_dist_complex/A1` / `001` — package **G**.
- [ ] `chamfer/equal_dist_sequence/C1` / `001` — package **G**.

## Full capability checklist

This covers all **50 capability families plus one reference topic** in `coverage.json`, including work outside the captured solids. The recorded status describes existing evidence, not whole-family completion. Even an `implemented-with-tests` family can have explicit remaining variation coverage. Each unchecked row requires evidence review and closure of the stated outcome, not automatically a rewrite or another test.

The catalog's suggested next cases can lag newer checkpoints. Reconcile them with current manifests before repeating historical work; for example, do not restart old H2/H4 or torus investigations solely because an older family note still names them. The required outcomes below remain the closure targets. See `coverage.json` for per-family evidence, variation axes, and detailed next-case suggestions.

- [ ] **F01 — Constant-radius planar fillets**. Package **C**. Recorded status: `implemented-with-tests`. Round selected planar-solid edges to the requested constant radius, for convex and concave material, while preserving a valid solid.
- [ ] **F02 — Variable radius along a tangent chain**. Package **F**. Recorded status: `partial`. Create a smooth blend whose radius follows the requested physical profile along an entire connected chain, including unequal-length and split edges.
- [ ] **F03 — Different radii meeting at a corner**. Package **F**. Recorded status: `unverified`. Create a valid joined corner where incident edge fillets have different specified radii, with the intended contacts and continuity.
- [ ] **F04 — Tapered radius reaching zero**. Package **F**. Recorded status: `partial`. Taper a blend continuously to zero radius at an endpoint without leaving an invalid seam or degenerate solid.
- [ ] **F05 — Fillet section representation choices**. Package **G**. Recorded status: `reference-only`. No independent representation-selector requirement. The resulting shape must satisfy geometric accuracy and downstream use regardless of its internal representation.
- [ ] **F06 — Smoothness within and between blend surfaces**. Package **G**. Recorded status: `partial`. Achieve the required smoothness within blend surfaces and across their joins; distinguish that from tangency to the support faces.
- [ ] **C01 — Equal-distance chamfers**. Package **E**. Recorded status: `implemented-with-tests`. Bevel selected edges by equal specified setbacks on both supports, including convex and concave material.
- [ ] **C02 — Unequal chamfer distances and side identity**. Package **E**. Recorded status: `implemented-with-tests`. Bevel with independently specified setbacks on the intended two support faces; orientation or storage order must not silently exchange them.
- [ ] **C03 — Distance-and-angle chamfers**. Package **E**. Recorded status: `unverified`. Produce a chamfer with the requested setback and angle, including non-right-angle supports, using any equivalent parameterization.
- [ ] **C04 — Constant-throat chamfers**. Package **E**. Recorded status: `unverified`. Produce a chamfer whose normal sections maintain the requested throat dimension along the selected edge.
- [ ] **C05 — Constant throat with penetration**. Package **E**. Recorded status: `unverified`. Produce the requested constant-throat chamfer with penetration, satisfying both the throat and offset-support constraints.
- [ ] **C06 — Meaning of distance on curved supports**. Package **E**. Recorded status: `behavior-difference`. Produce the intended measurable setbacks on curved supports after explicitly translating dimension conventions between kernels.
- [ ] **G01 — Plane/cylinder lips and rims**. Package **C**. Recorded status: `implemented-with-tests`. Blend lips and rims formed by planes and cylinders, for holes, bosses and channels in either material sense.
- [ ] **G02 — Plane/cone and drafted rims**. Package **C**. Recorded status: `partial`. Blend drafted or conical rims over varying cone angles and near the apex when a valid requested result exists.
- [ ] **G03 — Cylinder/cylinder, sphere, torus and spline support pairs**. Package **C**. Recorded status: `partial`. Blend intersections between curved supports, including cylinders, cones, spheres, tori and spline surfaces, with valid contacts and trimmed output.
- [ ] **G04 — Equivalent analytic versus NURBS representations**. Package **D**. Recorded status: `partial`. Handle equivalent physical geometry consistently when represented analytically, as exact NURBS, with split faces, or after STEP import.
- [ ] **G05 — Curved and closed spline spines**. Package **C**. Recorded status: `partial`. Blend along open and closed curved edges, including spline inflections and high curvature, with valid closure and contacts.
- [ ] **T01 — Tangent propagation across support changes**. Package **C**. Recorded status: `partial`. Round or bevel a tangent chain continuously across edge splits and changes of support face.
- [ ] **T02 — Closed chains and periodic seams**. Package **D**. Recorded status: `partial`. Complete a closed blend without gaps or invalid topology, regardless of where periodic seams lie.
- [ ] **T03 — Branching or ambiguous chain selection**. Package **H**. Recorded status: `partial`. Resolve or report ambiguous tangent branches predictably, while allowing the intended explicit path to be selected.
- [ ] **T04 — Open shells and free-boundary terminations**. Package **H**. Recorded status: `partial`. Blend open-shell edges and terminate correctly on free boundaries for surface-modeling tasks.
- [ ] **J01 — Single fillet with oblique end faces**. Package **C**. Recorded status: `implemented-with-tests`. Terminate a blend correctly against oblique end faces while preserving the requested radius and a valid boundary.
- [ ] **J02 — Single blend ending across multiple faces**. Package **C**. Recorded status: `partial`. Terminate one blend across a neighborhood of several end faces without gaps, overlaps or damage to unrelated geometry.
- [ ] **J03 — Three equal-radius edges with the same concavity**. Package **C**. Recorded status: `implemented-with-tests`. Join three equal-radius, same-concavity rounds at a valid corner over general planar support angles.
- [ ] **J04 — Two-edge junction with nonorthogonal supports**. Package **C**. Recorded status: `partial`. Join two selected edge rounds at an oblique corner with the required continuity and preservation of the remaining sharp edge.
- [ ] **J05 — Mixed convex/concave corner transitions**. Package **C**. Recorded status: `partial`. Construct valid transitions where convex and concave blends meet, including oblique support configurations.
- [ ] **J06 — Four or more interacting blends at a vertex**. Package **C**. Recorded status: `partial`. Join four or more interacting blends at a vertex when the requested configuration has a valid solution.
- [ ] **J07 — General curved multi-blend corner filling**. Package **C**. Recorded status: `partial`. Close multi-blend corners on general curved supports with valid contact and continuity, independent of patch construction method.
- [ ] **D01 — Contact collapse and singular tips**. Package **C**. Recorded status: `partial`. Handle valid limiting cases where contact curves collapse or tips become singular, with controlled rejection of impossible configurations.
- [ ] **D02 — Disappearing support faces**. Package **C**. Recorded status: `partial`. Continue valid blending through the disappearance of a support face and reconstruct the remaining material correctly.
- [ ] **D03 — Disappearing unselected edges and blend contacts**. Package **E**. Recorded status: `partial`. Continue valid blending when an unselected edge or intermediate blend contact disappears, rebuilding local topology correctly.
- [ ] **D04 — Folded offsets and overlapping blend patches**. Package **G**. Recorded status: `partial`. Produce a valid result where one exists as offsets fold or blend patches meet; reliably reject invalid or self-intersecting outcomes.
- [ ] **D05 — Contact crossing a trim loop or hole**. Package **G**. Recorded status: `partial`. Handle blend contacts that reach trim boundaries or holes, producing the intended material boundary or an accurate geometric rejection.
- [ ] **D06 — Distant face/cavity collisions and global self-intersections**. Package **G**. Recorded status: `partial`. Detect or resolve collisions with distant faces and cavities so a successful blend cannot silently leave an invalid or self-intersecting body.
- [ ] **N01 — Reliable contacts near singularities and competing solutions**. Package **G**. Recorded status: `partial`. Find the intended valid contact solution near singularities and competing branches without unstable branch jumps or false failures.
- [ ] **N02 — Accuracy through rapidly changing curvature**. Package **G**. Recorded status: `partial`. Maintain requested geometric accuracy through rapidly changing curvature and unequal feature scales without missed contact failures.
- [ ] **N03 — Pcurve domains, periodic parameters and seam endpoints**. Package **D**. Recorded status: `partial`. Keep blend boundaries on both supports and close them correctly when surface parameters, orientations or seam positions change.
- [ ] **N04 — Almost tangent and nearly coplanar edges**. Package **G**. Recorded status: `partial`. Handle nearly tangent and nearly coplanar selections consistently around tolerance thresholds, without invalid output or unstable propagation.
- [ ] **N05 — Near-limit sizes and tiny retained features**. Package **G**. Recorded status: `implemented-with-tests`. Retain valid tiny features near blend size limits and reject sizes beyond the valid geometric range consistently across transforms and scales.
- [ ] **W01 — Sequential fillet/chamfer composition**. Package **H**. Recorded status: `partial`. Apply successive rounds and bevels to earlier feature results with correct dimensions, valid topology and stable selections.
- [ ] **W02 — Simultaneous straight and curved selections**. Package **H**. Recorded status: `unverified`. Apply one requested selection containing straight and curved edges with the intended joined result, independent of selection enumeration order.
- [ ] **W03 — Blends on Boolean-generated intersections**. Package **H**. Recorded status: `implemented-with-tests`. Blend physically selected intersection edges created by Boolean operations and preserve valid solid geometry.
- [ ] **W04 — Stable references after topology edits**. Package **H**. Recorded status: `partial`. Keep downstream references attached to their intended geometry after upstream edits split, replace or remove blend topology.
- [ ] **W05 — Closed meshes, STEP and downstream modeling**. Package **H**. Recorded status: `implemented-with-tests`. Deliver blend results usable for closed meshing, STEP interchange and further modeling while preserving dimensions and validity.
- [ ] **W06 — Actionable failures and recovery**. Package **H**. Recorded status: `partial`. Explain failed selections and stages clearly, preserve the model and allow the user to correct or retry a failed blend.
- [ ] **W07 — Invalid selections, parameters and immutable failure**. Package **H**. Recorded status: `implemented-with-tests`. Handle invalid, empty, duplicate or stale selections and invalid numeric parameters deterministically without panic or input mutation.
- [ ] **W08 — Face-driven rolling ball and stopping faces**. Package **H**. Recorded status: `unverified`. Construct a round between selected support faces and terminate it at specified limiting faces, through any equivalent feature construction.
- [ ] **W09 — Boolean and blend composition**. Package **H**. Recorded status: `implemented-with-tests`. Fuse or cut bodies, then blend the intended resulting intersection edges while preserving the modeling intent through edits.
- [ ] **S01 — 2D line/arc fillets**. Package **I**. Recorded status: `partial`. Round trimmed planar line/arc corners to the requested radius and chosen side, retaining connected sketch geometry.
- [ ] **S02 — 2D general curves and multiple solutions**. Package **I**. Recorded status: `unverified`. Round general planar curve corners with the intended radius, trimmed domains and selected solution when several are possible.
- [ ] **S03 — 2D chamfers and corner edits**. Package **I**. Recorded status: `unverified`. Create and edit planar sketch chamfers with specified setbacks or angles, maintaining connectivity and edit behavior.

F05 is reference-only: close its review by confirming representation differences do not cause an unresolved accuracy, interoperability, downstream-modeling, or performance issue. Do not implement an OCCT-style selector just to match an API.

## Required validation and checkpoint procedure

For each implementation achievement:

- [ ] State the physical modeling task, intended dimensions, terminal behavior, valid size range, and independent correctness oracle.
- [ ] Add a meaningful regression that fails on the parent with identical source and fixtures. If the result contract intentionally changes, rewrite its test first with equally strong or stronger checks and explain why.
- [ ] Keep all existing assertions, validity guards, canonical thresholds, replay fitting budgets, and rejection behavior unless a deliberate separately proven contract change requires otherwise.
- [ ] Check topology, closed tessellation, independent material volume, requested radius/setback/angle, required tangent continuity, input immutability, history, and STEP through the common harness. For open shells use their boundary contract.
- [ ] Exercise relevant transforms, reversal, selection order, split/unsplit geometry, seam positions, support representations, and near-limit sizes. Record what is and is not covered.
- [ ] Run affected release suites, all-target Clippy, single-thread actions smoke, replay/capture checks, and required native checks. Record commands, logs, source hashes, known failures, warnings, and ignored tests.
- [ ] Build the production replay last and run original-scale targeted comparisons with it. No stale prototype binary as final proof.
- [ ] Commit implementation and proof as one stable achievement when authorized.
- [ ] Coordinator runs a fresh complete capture on clean committed source, verifies prior results, archives evidence, refreshes catalog, and commits the capture/report separately.
- [ ] Update this tracker, package state, and the activity log before handing back control.

Use `CARGO_BUILD_JOBS=12`. Format touched Rust only with `rustfmt +nightly --config skip_children=true`; do not run repository-wide `cargo fmt`. Use `truck-shapeops/tests/common/mod.rs` and the common blend STEP harness. Force-add intended ignored JSON/ZIP/STEP/BREP evidence. Check diffs excluding preserved native STEP/BREP bytes where needed. Do not commit logs with secrets or unrelated files.

Dependency test reuse requires identical relevant source hashes and explicit baseline limitations. The known geometry precision failure must stay documented, not be weakened to achieve a green report. Benchmark smoke only proves the benchmark runs; performance claims require measured before/after medians.

## Evidence and portability

All paths here are relative to the repository root unless linked otherwise.

| Evidence | Location |
| --- | --- |
| Current implementation report | [Mixed corner report](checkpoints/2026-10-22-mixed-curved-implementation/report.html) |
| Detailed implementation and F5 handoff | [Mixed corner handoff](checkpoints/2026-10-22-mixed-curved-implementation/HANDOFF.txt) |
| Parent proof, validation, source hashes, 76-member controls archive | [Mixed proof directory](../../truck-benchmarks/results/2026-10-22-mixed-curved-corners/REPORT.md) |
| Latest complete corpus summary | [Mixed capture report](capture/2026-10-24-mixed-curved-corners/REPORT.md) |
| Complete baseline operation records | [Torus manifest](capture/2026-10-22-torus-corners/results.json) |
| Current comparison and scope accounting | [Mixed comparison summary](capture/2026-10-24-mixed-curved-corners/comparison-summary.json) |
| Capability definitions and evidence | [Coverage](coverage.json) |
| Generated inventory and case navigation | [Catalog](catalog.html) and [viewer](viewer.html) |

Current mixed capture archive SHA256: `5f5b0bf951da9e2e3809c52b17aa5b72e7b1c177f2ddc3086002a8828052c6be` (8,497 members).

Baseline torus archive SHA256: `0bad14be0c383aa48050e7941d3311ca8665946be5e6db9f145d5e02e52ba9b8` (8,491 members).
Mixed proof archive SHA256: `ffd52c1c367d115c1c55e8002f940e2d7926fcc2fd895addf29dc8c50ee93d8e` (76 members).

The `target/` paths are **local, ignored scratch artifacts**, not guaranteed in a fresh clone or subagent worktree. Their existence was checked on 2026-10-24. Before moving to another machine, preserve the F5 source, test sources, oracle/derivation, timing logs, fixtures, and helper commands with hashes. A subagent on this machine can read the root's artifacts through absolute paths, but must copy only its scoped inputs into its own worktree and record their provenance. If those files are absent, reconstruct from committed fixtures/evidence; do not assume the F5 prototype was committed.

Do not confuse raw experiment logs, pre-refinement binaries, incomplete timed-out runs, or historical unsupported-case assertions with current production evidence. Earlier handoffs are historical snapshots and can contain superseded stop/resume instructions and obsolete case diagnoses.

## Completion criteria for the entire effort

- [ ] Every positive-solid corpus operation has a verified final disposition; construction failures, invalid inputs, timeouts, discrepancies, and agreed exceptions remain separate categories.
- [ ] All non-solid/shell/sketch and missing/skipped fixture scopes have been reviewed; unresolved or unavailable evidence is explicitly recorded rather than counted as success.
- [ ] All 50 capability families have current outcome evidence or an explicitly agreed remaining scope/limitation; reference topic F05 does not become an artificial API requirement.
- [ ] Geometric constraints, continuity, material membership, global self-intersection behavior, STEP, downstream modeling, stable references, and immutable failure are covered at the appropriate scope.
- [ ] Practical performance meets the agreed product needs with recorded measurements; unresolved F5 tessellation cost is addressed.
- [ ] Required application integration and sketch behavior are validated after any necessary scope authorization; consumer pins are changed only when explicitly requested.
- [ ] Final full capture, catalog, proof archives, HTML report, and this tracker agree on commits, counts, exceptions, and remaining limitations.
- [ ] Stable achievements are committed, working-tree status is reported, and no owned build/test/capture jobs are left running at final handoff.

No overall completion percentage or universal parity claim should be inferred from the sampled corpus. If full completion cannot be demonstrated, report the exact remaining outcomes and blockers instead of marking the whole task done.

## Progress log

Update the existing checkboxes and package states, then append one row per stable achievement. Keep actual full-run totals separate from targeted probe results.

| Date | Package | Commit or artifact | Achievement | Remaining work and next action |
| --- | --- | --- | --- | --- |
| 2026-10-22 | A | `9f97532a` | Torus full corpus archived: 480 construct, 437 limited passes | 122 unresolved positive solid operations |
| 2026-10-22 | A | `9e3ce08a` | Mixed F1/H7 implementation and proof committed | Full mixed capture pending; H7 discrepancy retained |
| 2026-10-24 | A | `TASK_TRACKER.md` | Full-work handoff, 122-operation baseline ledger, 51-family review list, and subagent packages prepared | Assign coordinator/workers; start A's capture when continuation is authorized |

| 2026-10-24 | A/B/D/G | Root `master` @ `9e3ce08a`; scratch evidence under `target/` | Resumed with three workers; 444 source hashes/replay verified; 13 capture tests pass; serial baseline running. F5 winding retries and STEP isoparameter endpoint conversion have concrete probe evidence. | Freeze root source/replay until capture completes; proofs and integration pending. |

| 2026-10-24 | A | `9e3ce08a` / [capture](capture/2026-10-24-mixed-curved-corners/REPORT.md) | Full mixed capture: 482 construct, 438 limited passes; 480 old successes exact; 8,497 archive members verified | 121 unresolved positive solids (60 construction, 7 import, 10 inconsistent, 44 discrepancies); F5/exact-knot candidate validation next |

Next handoff must state: current HEAD/branch, changed files, active jobs, completed packages, exact validation evidence, actual corpus totals, unresolved discrepancies, local-only artifacts, and the first reproducible next action.
