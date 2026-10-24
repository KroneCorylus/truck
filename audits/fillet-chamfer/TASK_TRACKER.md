# Fillet and chamfer tracker

Updated 2026-10-24. Repository `truck`, branch `master`. Goal: OCCT-level fillet/chamfer
capability for equivalent modeling tasks, judged by dimensions, validity, editing behavior and
latency, not by matching OCCT's API or face structure. Scope is the OCCT V7_9_3 inventory in
[coverage.json](coverage.json). The previous 530-line tracker, with the full history and package
plan, is in git at `834ee9f5`.

## Current numbers

Last complete capture: `5a4e3bc6`, 396 cases / 648 operations
([report](capture/2026-10-24-exact-knots/REPORT.md)).

| Measure | Full run | After F5 (targeted, not yet in a full run) |
| --- | ---: | ---: |
| Constructs | 482 | 483 |
| Limited comparison passes | 438 | 438 |
| Unresolved positive solids | 121 | 121 |
| — construction failures | 64 | 63 |
| — solid import failures | 7 | 7 |
| — inconsistent inputs | 6 | 6 |
| — comparison discrepancies | 44 | 45 |
| No-solid captures (scope review) | 88 | 88 |

"Limited comparison" = valid output plus sampled two-way distance ≤ 0.001. It is not a global
error bound. A separate nine-case capture ([cohort](capture/2026-10-24-nine-case-cohort/REPORT.md))
adds `blend/complex/D2` (construction failure) and 8 no-solid cases.

Known, accepted test state: one ignored test, two existing Clippy warnings in shapeops tests,
and the `pcurve_derivation` precision failure in truck-geometry.

## Open work

Owners are suggestions. Case keys are `native case / capture number`; the number is `001` unless shown.

**Curved fillet junctions** (19)
`blend/simple/{F9,G4,G8,H3,H5,H9,I2,I4,I8}`, `blend/simple/X1/002`,
`blend/tolblend_simple/{D3,D5}`, `bugs/modalg_4/bug828`, `bugs/modalg_5/bug25657_1`,
`bugs/modalg_7/bug30940_{2,4,5,6}`, `blend/complex/G9/002` (seam/contact, overlaps imports).

**Remote two-edge junctions** (3): `blend/simple/{G2,G6,H1}`. A G2 candidate from before F5 is
in `target/b-g2-candidate/`. It is uncompiled and must be rebased onto the F5 commit.

**Unequal radii** (6): `blend/simple/{A4,P9}`, `blend/tolblend_simple/A2`,
`feat/featlf/{E2,E3}`, `blend/complex/D2`.

**Chamfer construction** (35)
- `chamfer/{equal_dist,dist_dist,dist_angle}_complex/{A4..A9,C2}` (21). These likely share a
  cause; investigate them together first.
- `chamfer/equal_dist_complex/C1`.
- `chamfer/{equal_dist,dist_dist,dist_angle}_sequence/C2` 005 and 010 (6): a 0.5 chamfer
  consumes an earlier 0.2 chamfer. An unexecuted regression with an analytic oracle is in
  `target/d-c2-consumed-contour-regression.rs`.
- `chamfer/{dist_dist,equal_dist}_sequence/C1` 002, 003 (4).
- `chamfer/dist_dist_sequence/C1` 007, 008 and `chamfer/equal_dist_sequence/C1` 006, 007 (4).
  The inputs are now consistent; construction fails in `trim_faces`.

**Imports and inputs** (13)
- Import failures: `chamfer/dist_angle_sequence/C1` 002–008.
- Inconsistent inputs: `chamfer/dist_dist_sequence/C1` 004–006 and
  `chamfer/equal_dist_sequence/C1` 004, 005, 008.

Fix by reconstructing geometry, never by loosening consistency tolerances.

**Discrepancies** (45). These construct but fail the 0.001 comparison. An initial
classification is in
[g-discrepancy-review](checkpoints/2026-10-24-mixed-curved-corners/g-discrepancy-review-2026-10-24.txt).
- Blends: `blend/simple/{C2,C6,C8,D1,D9,E4,E6,E8,F3,F5,G1,G5,H4,H7,J6,J8,K2,K3,K4}`,
  `blend/tolblend_simple/A1`.
- Chamfers: `chamfer/dist_dist/{B3,B4,C1,C2,C3,C4,C6,C7,D1,D2,D5,D6,D9,E1}`,
  `chamfer/dist_angle/A2`, `chamfer/{dist_angle,dist_dist,equal_dist}_complex/A1`,
  `chamfer/{dist_angle,dist_dist,equal_dist}_sequence/C1/001`,
  `chamfer/dist_angle_sequence/{A6,A8}` 005 and 006.

Some discrepancies may come from the reference rather than Truck. F5 is an example: Truck's
volume matches an independent oracle to 6e-9, while OCCT's is off by 2.5 at a 0.022 face
tolerance. Record such an exception separately, with evidence; do not count it as a pass.

**Outside the solid corpus**
- No-solid scope (96): shell/open-surface tasks, including the eight cohort cases
  `blend/simple/{Q9,R1,R2,R3,R5,R6,U9,V7}`. Decide which are in scope.
- Capability families: 50 plus reference topic F05 in `coverage.json`; none is closed. Never
  exercised: F03 (different radii at a corner), C03–C05 (distance-angle and throat chamfers),
  W02 (mixed straight/curved selection), W08 (face-driven rolling ball), S02 and S03 (sketch
  curves and chamfers).
- App work in `test_cad` (sketch fillets/chamfers, references, diagnostics) needs explicit
  authorization before any app or pin change.

**F5 follow-ups**
- The extrusion branch samples blend orientation at chart point (0.5, 0.5). On short bodies
  that point is outside the fitted domain (span ≈ 0.17 at height 0.13). Short-body tests at
  heights 0.13 and 0.15 still pass. Fix it once a failing case exists.
- Original-scale F5 replay: 11.4 s wall time and 1.7 GB peak memory. It is no longer the
  76 s tessellation hotspot, but it needs a benchmark median before any speed claim.
- A finite-facet mesher accuracy rewrite is parked on local branch `f5-extrusion-candidate`
  (`6df3e1d7`). F5 does not need it, and it changes existing replay meshes: the tilted-cap
  exact volume and curved-terminal closure tests fail. Only revisit it as its own mesher
  change.

## Rules for each fix

1. Add a test that fails on the parent with identical fixtures. Use an independent oracle
   (analytic or flux volume), not OCCT output. Never loosen existing assertions.
2. Run `cargo test --release -p <affected crates>`, the `replay_occt_blend` example tests,
   all-target Clippy, and `cargo bench -p truck-benchmarks --bench actions -- --test`.
   Use `CARGO_BUILD_JOBS=12` and run one Cargo job at a time.
3. Replay the targeted capture at original scale with the release replay binary, then compare
   it with `target/occt-distance-tree-plugin/compare_shapes`.
4. Commit the fix with a `CHANGELOG.md` line. Update the numbers above as targeted results.
5. Run a full capture at milestones, not after every fix. Use one worker and a 120 s timeout;
   four workers ran out of memory. The output directory must not exist.

```bash
python3 audits/fillet-chamfer/capture/run_capture.py \
  --occt ../investigations/sequential-holes-2026-10-10/occt \
  --library-path target/occt-sdk/usr/lib64 --plugin target/occt-distance-tree-plugin \
  --replay target/release/examples/replay_occt_blend \
  --data-path target/occt-public-chamfer-data \
  --output target/occt-capture-<name> --workers 1 --timeout 120
```

`DRAWEXE` must be on PATH. The native OCCT checkout has three intentional local modifications.

## Log

| Date | Commit | Result |
| --- | --- | --- |
| 2026-10-22 | `9f97532a` | Torus corners; full run 480 construct / 437 pass |
| 2026-10-22 | `9e3ce08a` | Mixed curved corners; F1 passes, H7 constructs |
| 2026-10-24 | `5a4e3bc6` | Exact knots and STEP masters; full run 482 / 438 |
| 2026-10-24 | `834ee9f5` | Nine skipped native cases captured as a separate cohort |
| 2026-10-24 | `d3858852` | Elliptical extrusion corners; F5 constructs (discrepancy 0.107, OCCT reference suspect) |
