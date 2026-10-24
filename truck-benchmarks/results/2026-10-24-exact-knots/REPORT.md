# Exact spline parameters and declared STEP masters

Status: **validated implementation checkpoint**, based on `96ca84d1c2ac0f49b2b40e7d58615eb5ee0e2988`. No new full-corpus outcomes are claimed here. The last complete corpus remains 482 constructions, 438 limited passes and 121 unresolved positive solids.

## Behavior

Numerically distinct knots no longer merge under the geometric tolerance. Multiplicity, knot grouping, cuts and removal coefficients use exact parameter equality/zero, preserving small parameter domains and clustered spans. Geometry tolerances remain unchanged. Checked constructors reject nonrepresentable knot arithmetic using conservative first/second derivative coefficient bounds over degree-wide supports; normalization rejects collisions atomically.

Checked basis evaluation reports nonfinite parameters or coefficients. Infallible evaluation retains IEEE arithmetic results rather than introducing arithmetic-overflow panics inside generic geometry evaluation. Structural zero-range and insufficient-knot errors still panic. These are bounded contracts: unchecked construction, edits/transforms, arbitrary control-point magnitudes and arbitrary derivative orders are not universally finite or panic-free.

The generic Newton search callbacks reject nonfinite parameters before evaluating geometry. This is not a claim that every decorator-specific solver or custom parametric implementation is guarded.

STEP conversion prefers a declared S1/S2 nonrational 2D B-spline pcurve master on a supported extrusion when its spatial representation is also a spline. Supported extrusion bases are line, ellipse, B-spline and rational B-spline. Analytic spatial curves retain their canonical representation. Edge conversion retains spatial fallback for closed endpoints or failed master trimming. Rational 2D masters and other supports retain prior behavior. Imported raw curves can now be `Curve3D::PCurve`; the sibling application's source review found no direct affected variant consumers. No application source or dependency pin changed.

## Regression proof

Twenty-three new behavior regressions fail at runtime on clean parent production source. Two additional tests exercise candidate-only error variants, and one checks identical finite basis values plus preserved structural errors. Assertions are unchanged between parent and candidate runs. The independent Bernstein derivative oracle covers polynomial and rational cases; STEP checks retain positions, first/second derivatives and parameter domain. Fixtures already committed in the repository remain unchanged.

See `identical-parent-tests.json`, `nonfinite-search-proof.json`, `basis-contract-proof.json`, the parent logs and `independent-review.txt`. The initial strict arithmetic wrapper exposed the unchanged R10 consumed-contact rejection; `r10-basis-backtrace.txt` locates it in rolling-ball contact cross-point search. `basis-contract-before.txt` records both new contract failures on that intermediate candidate. `basis-contract-fixed.txt` records their correction and all eight unchanged R10 tests passing.

The initial complete shapeops command produced 463 passes, one failure and one existing ignore. It excluded doctests and is **superseded diagnostic evidence**, not final acceptance. Its only failure was the corrected R10 panic. Preserve this history instead of replacing its log.

## Validation

All commands use `CARGO_BUILD_JOBS=12` and `RAYON_NUM_THREADS=1`. Log-adjacent JSON records exact commands, timing and exit status. `final-candidate-source.json` records the corrected source snapshot.

| Check | Current result |
| --- | --- |
| Final geometry/geotrait/modeling/STEP suites | 367 pass, one known `pcurve_derivation` precision failure, one existing ignore |
| Their doctests | 172 pass, three existing ignores |
| Focused basis/error/R10 correction | 13 pass |
| Final mesher/shapeops suites and doctests | 545 tests and 28 doctests pass; two existing test ignores |
| Corrected candidate replay / Clippy / actions smoke | 76 replay tests pass; Clippy and actions smoke pass (two existing test warnings) |
| Matched benchmark medians | Fixed-core alternating runs retain +2.88% / +3.35% finest-loft overhead; cap/sphere approximately unchanged across run averages |
| Final cleaned-source C1 replay | 24 captures, 144 native artifacts verified; four input-consistency improvements, no new successful operations |
| Fresh complete corpus | Pending after an accepted implementation commit |

The known pcurve failure compares derivatives near magnitude 2.83e9 with an absolute difference around 1.4e-6; its assertion and persisted counterexample are preserved. This is not an all-green workspace claim.

F5 production, test and example additions are preserved separately under `target/f5-paused-for-d/`. Pending F5/G2/C2 prototypes are not part of this candidate's accepted capability. The benchmark-only controls in the working tree are separately identified in their logs and must be preserved with their source when recording final medians.

Reproduction note: the archived `c1-final-replay.py` is the exact runner source. Copy it to `target/d-c1-final-replay.py` before invoking it from the repository root; it intentionally resolves its root relative to that location. The parent-proof helper likewise runs from the repository root and preserves modified production files with a `finally` restoration.

The remaining C1 reconstruction question is separate: `c1-step-uncertainty-review.txt` records representation-level STEP uncertainty and native per-entity budgets lost during export. Some proposed vertex movements exceed the STEP-declared uncertainty. This candidate neither moves those vertices nor introduces an implicit repair tolerance.

The full 545-test/28-doctest downstream run and 76-test replay run precede an ordinary `#[inline]` annotation on the shared basis helper. Their exact source snapshot remains `final-candidate-source.json`; post-inline checks are recorded separately. No algorithm or arithmetic expression changed in that experiment.

The first C1 production run has three successes, seven import failures, eight construction failures and six inconsistent inputs. The four changed captures (`dist_dist_sequence/C1/007–008`, `equal_dist_sequence/C1/006–007`) now reach construction and fail at `trim_faces`; their inputs remain unchanged. All three successful captures preserve the baseline comparison records exactly. None passes the limited comparison. The cleaned-source production rebuild and C1 replay confirm those same outcomes; the final archive and executable hash are in `validation.json`.

## Final annotated-source checks

The cleaned post-inline source passes 81 mesher tests (one existing ignore), 21 mesher doctests, all eight R10 tests and all 76 replay tests. Dependency suites retain 367 passes, one known failure and one ignore; 172 dependency doctests pass with three existing ignores. All-target Clippy and the unmodified actions benchmark smoke pass. The prior complete shapeops run remains explicitly tied to its pre-inline source. `post-inline-source.json`, `production-build-provenance.json`, the log sidecars and `validation.json` bind the final checks to actual files and the production replay.

Pinned seven-sample alternating comparisons preserve the modest finest-loft slowdown: parent 111.3/113.6 ms, candidate 114.5/117.4 ms. The cap averages are approximately unchanged, and its legacy accuracy limitation is not resolved by this work. Unpinned timing variation is preserved in the logs; initial direct binary invocations without `--bench` are smoke-only and excluded from timing claims. See `benchmark-review.json` for every fixed-core median and `benchmark-control-sources.zip` for the exact benchmark module/fixtures. This checkpoint does not claim zero performance regression.

`c1-clean-production-evidence.zip` contains the full final targeted run and verifies every archived byte. `c1-pre-inline-evidence.zip` preserves the earlier snapshot. `validation-runners.zip` contains the exact local reproduction helpers. The new complete corpus remains pending and must run on the committed clean source before updating corpus counts.
