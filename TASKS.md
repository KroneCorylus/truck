# Open work

The single list of open kernel work and how to split it across agents. Read
[AGENTS.md](AGENTS.md) first; its rules win over anything here. Updated 2026-11-05.

The main program is OCCT-level fillet/chamfer capability: dimensions, validity, editing
behavior and latency on equivalent modeling tasks, not matching OCCT's API or face structure.
Scope is the OCCT V7_9_3 inventory in
[audits/fillet-chamfer/coverage.json](audits/fillet-chamfer/coverage.json). Case keys below
are `native case/capture`; the capture is `001` unless shown.

## State

Work resumed at the user's request on 2026-11-05 from Truck d2d7be32 and app c27269d.
Both working trees were clean on entry. Continue until asked to stop; no new commit,
push or PR is authorized. The previous stop request is superseded for ongoing work.
Current bounded streams:
- I2: derivative/control bounds and independent material oracle added; prospective
  revolved_cone_junction capability test is deliberately red on the frozen baseline.
  Physical shared endpoints, CI UV and positive-weight canal control construction now
  exist in independent Python probes. All seven paired analytic traces now have
  bounded ledgers, but candidate coefficients/controller differ from oracle controls;
  circle construction and runtime trig terms remain unbound. Rust builder must bind
  actual controls and evaluation before consuming those analytical budgets;
  production admission stays disabled until complete budgets and tests pass. Evidence:
  target/i2-implementation-20261105/REPORT.txt and
  target/i2-endpoint-trace-20261105/REPORT.txt and
  target/i2-paired-trace-20261105-a/REPORT.txt. Do not claim a green full workspace;
  C2-scoped validation explicitly excludes this unfinished integration target.
- C2 admission corrections implemented: single-shell scope, retained-plane residuals,
  cap signs and conic endpoints. Independent correction review approved both blockers
  resolved, with no essential remaining defect in the five-file delta. Five new
  rejection tests fail isolated d2d7be32; 15 focused and 630 affected tests pass.
  Full486 remains 367/345 with no lost checks. Evidence:
  target/c2-admission-fix-20261105/results.json. I2 integration remains excluded/red.
- DD/DA complex C2 contract reviewed: actual cases fit the existing family, subject
  to the corrections and numerical/branch acceptance obligations recorded below.
  Both actual cases share labelled cap/axial intercept pairs along the tangent chain,
  so no new transition surface is proposed. Evidence:
  target/c2-unequal-contract-20261105/contract.txt. Admission fixes approved; bounded
  unequal candidate now validates both actual targets. Full486 records 369/347,
  exactly two gains and no lost checks; 30 sequence and three preference controls
  remain passing. Final focused tests: 21; library: 98; integration: 541 plus one
  skip; seven docs and 77 replay tests pass. Native exact QA and 13,280 independent
  material witness positions pass. Independent unequal-construction review gives
  bounded approval with no blocking findings; all 164 stored-conic certificates and
  3,981 evidence hashes independently checked. Latency remains unmeasured.
  Evidence: target/c2-unequal-validation-20261105/results.json. This is scoped
  validation, not a globally frozen checkout or performance claim.
I2 private builder remains disabled in production. Stored derivative-control and
cone-transform coefficient corrections now compile and pass seven private tests,
including material/STEP, and all-target release Clippy. CS uses an affine UV PCurve
on the actual canal. Evidence: target/i2-certificate-corrections-20261105/REPORT.txt.
Physical endpoint boxes/sliver bounds and CI stationary isolation with signed
separation now pass eight private tests including three radii/material/STEP; Clippy
and formatting pass. Evidence: target/i2-paired-runtime-gate-20261105/REPORT.txt.
Actual paired UV compositions and support/trig evaluation allowances remain unfinished;
separation charges are provisional until those terms are included. The prospective
public test remains unchanged/red and admission disabled. No geometric obstruction
was found. All processes joined and builds released. Freeze production/tests/harness
sources for C2 fresh build/smoke/material work; that worker now owns the heavy slot.
I2 composition implementation may proceed only in new scratch patch artifacts during
this freeze, without Cargo/rustc or changes to campaign-bound repository sources.
I2 scratch UV proposal is preserved but NOT ready to apply:
target/i2-uv-composition-20261105/REPORT.txt. Adapter samples pass, but paired
support composition, chart endpoint allowances and evaluation terms remain absent.
Next bounded work must compute actual coefficient-based K/F in K + F*epsilon_trig
for every pair, complete nontrig terms and state the evaluator contract explicitly;
no arbitrary epsilon or nominal factor substitutes for that analysis. Live I2 source
remains unchanged during C2 validation; new I2 work stays scratch-only.
I2 support composition produced 42 conditional K/F ledgers in
 target/i2-support-composition-20261105/REPORT.txt, but the scratch patch rejects all
frames/profiles except exact retained synthetic fixture constants. This is research
corroboration, NOT an acceptable general admission implementation; do not apply as such.
Rust trig precision remains an explicit unresolved external assumption. Independent
review must assess ledger soundness, remove fixture-only guards in the design and
identify the smallest justified numerical contract without speculative global rewrites.
C2 matched benchmark preparation is complete: 14 cases, unchanged adapters and
23 lightweight integrity tests. Evidence: target/c2-benchmark-prep-20261105/REPORT.txt.
Independent harness review found an optional-input inventory hole. Coordinator fixed
the shared identity check to require the exact prepared directory inventory, binding
preference-file absence as well as presence. Three new regressions fail archived incoming
code; all 26 lightweight integrity tests pass. Evidence:
target/c2-input-inventory-fix-20261105/. No builds or timings ran for the correction. C2 fresh build/smoke now owns the build
slot. Both-engine smoke/material approval and serialized timings
remain required; no historical approval substitutes for the new campaign.
C2 fresh smoke found exact serialization repeatability failures in all three modes:
chamfer_extrusion.rs starts each output wire from HashMap::values().next(). Campaign
remains failed; preserve its material/QA evidence without timing approval. Worker will
finish bounded evidence, join processes and release its freeze/build slot. Coordinator
then fixes deterministic wire ordering with a parent-failing regression before a new
exclusive campaign; do not weaken repeatability or reuse this failed approval.
C2 campaign finished failed and released its freeze/build slot. Coordinator replaced
hash-based wire starts with earliest surviving construction-order edge. New exact
repeatability regression fails incoming source; 22 focused STEP tests, scoped Clippy
and nightly formatting pass. Evidence: target/c2-wire-order-fix-20261105/results.json.
Fresh fixed-source validation/smoke remains pending. G4 Truck/OCCT STEP and G8 Truck
STEP failed exact QA in the frozen campaign despite original/fresh native BREPs passing;
attribute these export/import failures independently, preserve failed statuses.
G4/G8 STEP attribution: target/g4-g8-step-qa-20261105/REPORT.txt. Historical and
current Truck exports import to byte-identical BREPs. Historical G4 output-check
set exact mode after construction/Init, so its reported exact success was ordinary QA;
that evidence must not be cited as exported exact validity. Current proper checks pass
all original/fresh native BREPs. Failures localize to reconstructed ellipse pcurves,
Truck exported spline/cylinder residuals, and G4 OCCT STEP lowered edge tolerance.
Raw Truck versus prepare_for_step ownership still needs frozen raw/prepared dumps.
Preserve failures; no geometry, tolerance, QA or timing waiver follows from material passes.
Deterministic C2 validation completed: target/c2-wire-order-validation-20261105/REPORT.txt.
644 scoped tests/one skip, 22 focused STEP, seven docs, 77 replay tests and all-target
Clippy pass; full486 retains 369/347 with no lost checks. All 14 both-engine smoke cases
now pass exact serialization; all 28 material rows pass. Three exported G4/G8 STEP
exact-QA failures remain; 12 cases are individually eligible but no timing approved.
All processes joined; build slot released to frozen raw/prepared G4/G8 attribution.
No production/test/harness source edits during that diagnostic capture. I2 scratch
composition review proceeds independently; public prospective test remains red.
Raw/prepared G4/G8 attribution completed (target/g4-g8-raw-prepared-20261105/REPORT.txt).
G4 spatial residual already exists in its Hermite trim; preparation is byte-identical.
G8 raw intersection agrees with the cylinder, but prepare_curve's procedural cubic fit
introduces the residual; serialization preserves coefficients. Both errors fit their
current approximation contract, so correction must add an explicit stricter support
agreement contract with parent-failing regressions, not claim old budget violation.
Separate ellipse/import-pcurve and G4 OCCT round-trip failures remain unresolved.
Build slot is released to bounded G4/G8 spatial support corrections; I2 typed-composition
and supported-libm analysis may proceed independently without heavy builds.
I2 review (target/i2-composition-review-20261105/REPORT.txt) rejects literal fixture
admission, identifies Python derivative-domain sliver omission, and supplies typed
Rodrigues coefficient/frame formulas. Fix endpoint convex-hull coverage; conditional
Python rows are not proof of uncompiled Rust controls. Do not claim stored stationary
uniqueness from analytic CI isolation; use uniform position bounds for minimum value.
I2 typed scratch proposal remains incomplete (target/i2-typed-composition-20261105/REPORT.txt).
Typed harmonic/dual-frame extraction and endpoint hull corrections exist; finish rational
extrapolation denominator/derivative bounds, actual affine intermediate error, plane
projection magnification, chart reversal tests and typed ledger bridge before application.
Coordinator inspected live i2.rs drift 9d5bf019 -> d81cf69c: whitespace-only function
formatting, not a semantic change. Preserve/disclose it; no source cleanup during builds.
I2 libm source analysis completed: target/i2-libm-contract-20261105/REPORT.txt.
Matching Fedora source/installed IFUNC variants yield claimed conditional scalar
sin/cos/sincos error <6.074e-14 on [-4,4], conservative contract 2^-40. Independent
numerical/runtime review pending. Not a generic Rust guarantee; final candidate's
scalar lowering, numerical environment and actual argument hulls must satisfy the
stated contract. No machine-hash geometric gate or public I2 admission authorized.
G4/G8 spatial support fixes completed (target/g4-g8-support-fix-20261105/REPORT.txt):
G4 whole-span trim budget capped at 5e-8; eligible G8 procedural export fits check
both represented extrusions. Parent regressions fail and targeted spatial exact-QA
pairs now pass. 244 scoped tests/13 docs pass; material witnesses remain passing.
Independent review pending. Three ellipse/pcurve pairs per Truck case and G4 OCCT
round-trip failure still block whole STEP QA. New g8-support-curve.json fixture is
ignored by *.json and must be explicitly included if a later commit is authorized.
I2 libm review gives bounded numerical approval, including independent compensation,
FMA/tie and reduction arguments (target/i2-libm-review-20261105/REPORT.txt).
Actual final argument hulls, scalar lowering and folded constants remain integration
requirements. I2 closure worker now has permission to apply its owned patch and the
exclusive build slot, max eight jobs. Public admission remains disabled pending full
executed gate and independent review. Other work is read-only during this build.
Spatial support review requests two general-contract fixes: final assembled spline
must be rechecked after endpoint replacement, and eligible guide reuse must pass the
same support gate. G4 cap and recorded G8 fixture correction have bounded approval;
whole STEP still failed. Coordinator applied the return-path fixes and guide/join
regressions: both fail incoming behavior, 135 STEP tests and six doctests pass,
release all-target Clippy and scoped nightly formatting pass. Final joined coefficients
are checked after endpoint mutation; eligible guide returns use the same support gate.
Evidence: target/step-support-final-validation-fix-20261105/results.json. Reader precision
wording is corrected; no new replay or performance claim for this narrow delta.
Ellipse export research demonstrates bounded UV PCURVEs referencing existing face
surface identities; scratch outputs pass paired exact QA and material checks. Production
writer/numerical-budget implementation is in scratch while I2 owns the build slot.
Evidence: target/g4-g8-ellipse-contract-20261105scratch/REPORT.txt. The separate G4 native
STEP tolerance-loss failure remains unresolved.
I2 closure passed 16 private library and 18 integration/material/STEP tests, Clippy,
formatting and runtime mapping checks, with public positive still red. Evidence audit
then found reversal argument-formation magnitude must include absolute range endpoints.
The phase-formation correction now passes its parent-failing mechanism regression,
17 private library and 19 integration/material/STEP tests, Clippy and formatting.
Final source 40886121 and executable 994a41de bind 70 actual Rust UV rows and runtime
mapping/state evidence. See target/i2-composition-closure-20261105/REPORT.txt.
Complete-gate review blocks public admission: source profile recognition is incomplete,
only seven of 18 output edges have paired certificates, the reviewed runtime epsilon
is not enforced by construction, and expanded canal fit error uses an unjustified
factor two. Evidence: target/i2-complete-gate-review-20261105/REPORT.txt.
Coordinator applied the typed runtime-budget enforcement: the mechanism regression
fails incoming behavior, and 18 private library plus 20 integration/material/STEP tests
pass; release all-target Clippy and scoped formatting pass. Evidence:
 target/i2-runtime-budget-enforcement-20261105/results.json, final source 42a6c517.
The initial parent harness compile failure is preserved, not counted as proof.
Deployment eligibility remains separate. Complete the three geometric certificate gaps
in scratch, preserving all thresholds and evidence; the heavy slot is now released.
Public admission stays disabled and the prospective positive test stays red.
Ellipse writer candidate is incomplete and admission-disabled in
 target/g4-g8-ellipse-writer-20261105. Continue numerical closure and type/graph validation;
its diagnostic stored-control checks do not establish extraction or evaluator bounds.
Extended scalar trig contract now has independent bounded approval on [-8,8]:
per-component error <=1.778973623518272e-15 <2^-48 under the recorded implementation
and FP assumptions. Evidence: target/ellipse-libm-contract-20261105/REPORT.txt and
 target/ellipse-libm-review-20261105/REPORT.txt. Research-description correction:
JSON selection windows are conservative supersets, not threshold-clipped intervals;
the proof uses a global reduction enclosure, so this does not change the bound.
Final writer/OCCT mapping, actual argument hulls/formation error, serialized coefficients
and complete evaluation budgets remain required. This does not widen I2's contract
or authorize geometry admission by itself.
No concurrent timing campaign. Coordinator owns this
tracker, CHANGELOG, combined validation and subsequent serialized measurements.

Committed equal-C2 evidence: target/c2-constructor-results.json and c2-constructor-delta.patch.
The release suite passed 623 tests with one existing skip; nine final focused tests,
seven doctests, 77 replay tests, Clippy, formatting and actions smoke passed. Full486
now records 367 constructions / 345 limited passes, with no lost checks. All 30 C2
sequence controls remain passing. The equal-distance complex C2 gain passes fresh
native QA and 3,412 independent material witnesses. C2 latency has not been measured.
App changes are committed as c27269d; their files match the validated app snapshot.
I2 remains a private candidate with shared-trim numerical budgets unresolved. Existing
native comparison failures and neighboring performance uncertainty remain open.
The dated entries below retain historical evidence; current scope is the bounded streams above.

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

## Resumed — 2026-11-03

Matched benchmark harnesses are prepared in truck-benchmarks/{examples,occt,scripts}/
nov3_matched.* for ten H1/F9/P9/A1/A2 configurations. All ten Truck smoke operations pass
consistency, repeatability and input preservation. Nine OCCT operations pass exact QA and
capture identity. F9 fresh native matches its capture, but both original and fresh native
BREPs fail exact BRepCheck while ordinary QA passes; its validated timing remains blocked.
Do not weaken QA or time an unmatched/invalid result as an equivalent valid operation.
Harness timing boundaries received review approval. Coordinator fixed the two script
blockers: smoke approval binds complete build/runtime/input identity; stages and outputs
are exclusive, and identity is rechecked before summary publication. Six integrity tests
pass, including three that fail the incoming script. Failed attempts remain preserved;
new attempts require a fresh campaign directory. Evidence:
target/nov3-benchmark-review-fixes-20261103/validation.json. Fresh build/smoke is required.
Bounded F9 native-QA attribution is complete: edge 2 / cylinder face 3 has
InvalidCurveOnSurface and InvalidSameParameterFlag. Direct dense evaluation corroborates
the exact discrepancy 0.00557657 exceeding the declared edge tolerance 0.00538166;
ordinary sampling misses the peak. All three tighter configurations also fail.
No validated native alternative was found. Keep default F9 matched timing blocked;
any necessary repair must pass unchanged QA/material checks and stay inside construction
timing. This native BREP issue is separate from Truck's imported classifier failures.
Evidence: target/f9-native-qa-20261103/REPORT.txt. Original artifacts remain hash-identical.
No performance measurements yet. Harness evidence and later serialized commands:
target/new-path-benchmark-prep-20261103/REPORT.txt and commands.txt.


A2 triangle family: four DD/DA A7/A9 capture gains, with the recorded replay at
362 constructions / 340 limited passes and no lost checks. The review found a
translation-dependent admission tolerance. Coordinator reproduced the bad gate at
translation 1e12 with axis intercept 0.199 instead of 0.2, then moved intercept and
coincidence checks into the local setback frame. The same unit regression now passes;
five integration tests preserve ordinary translated acceptance and immutable rejection
of perturbed or numerically extreme inputs. Face/selection order coverage is explicitly
not reversed-shell coverage. Final checks passed: 98 focused tests, release Clippy (known warnings), smoke, scoped
formatting, and all four target captures at unchanged comparison tolerances.
Evidence: target/a2-triangle-gate-20261103/; original capability evidence and frozen-parent
proof: target/a2-triangle-family-20261103/. Performance and stable combined validation
remain pending. All four incident faces at each new contact are checked.

P9 implementation completed: one construction and limited comparison gain, maximum
sampled distance 1.84e-8. Independent volume 123.69042579279512; 2,116 durable oracle
witnesses and 2,124 retained contract witnesses pass. Recorded validation: 556 tests plus
one skip, six focused tests, seven doctests, 77 replay tests, Clippy, smoke and formatting.
Both new tests fail on frozen d89c7368; all 4,367 archived files remain unchanged.
Scope is unsplit rectangular boxes, shared crease seam, residual Q–R arc and surviving
sharp vertical edge. Review found two blockers. Coordinator fixed rational straight-edge
admission: finite nonzero same-sign weights, finite projections, and a finite nonempty
clamped domain are now required. The opposite-sign public-API reproduction timed out
in recursive curve subdivision before the fix; it now rejects without mutation, while
positive/negative same-sign controls pass. Seven focused tests, Clippy and formatting
pass. Evidence: target/p9-review-fixes-20261103/weights-validation.json.

P9 numerical-budget and weight corrections received independent bounded approval.
Both original blockers are resolved; no demonstrated blocking gap remains in P9.
The incoming production spline reproduced 5.9497538896e-8 error at t=25/53 against an
80-digit cylinder reference, exceeding its unchanged 1e-8 target. Both final private
regressions fail incoming production. Half the unchanged target now covers conservative
frame/construction/evaluation error; half covers Hermite truncation on stored knot spans.
This is FP forward-error analysis, not formal interval certification. Translation and
frame guard pairs plus public immutable rejection are tested; the rational-weight fix
and regression remain unchanged. Ten focused tests, seven doctests, scoped Clippy,
formatting, smoke, P9 replay and both material-oracle sets pass. P9 remains a limited
comparison pass. Evidence: target/p9-numerical-budget-20261103/validation-results.json.
Stable combined validation and serialized benchmarks remain pending.


F9 mesher correction is implemented in triangulation.rs and its focused unit tests:
reciprocal average division spacing replaces tangent-length scaling only for the existing
single-direction refinement path. Geometric sampling, error checks, winding, tolerances
and termination remain unchanged. The bounded old-code regression fails its vertex budget;
36 chart/anisotropy controls pass with analytic geometry and coverage checks. Validation:
83 meshalgo tests plus existing skip, 21 doctests, Clippy, formatting and six F9 tests.
Real .001 F9 replay now completes closed without debugger overrides, with 26,328 KiB
verified peak RSS. Evidence: target/f9-mesh-metric-20261103/results.json.

Mesher delta received independent bounded approval: spacing arithmetic, synthetic-grid
ordering, metric/UV separation and unchanged physical refinement checks were verified.
This does not establish whole-pipeline or extreme-chart invariance. Final acceptance
remains held for a G2 resource correction, stable combined validation and serial benchmarks.
Matched G2 controls now establish pre-existing growth: same input, one worker, CPU affinity,
90 s wall limit, 3 GiB address-space and sampled 1.5 GiB RSS ceilings. Isolated metric
parent/current diagnostic builds serialize identical constructed solids. Both finish all
face tessellation; the metric parent stops during mesh merging, current reaches attribute
welding. Current and parent two-direction fillet grids match. The new metric reduces
single-direction refinement; it does not cause the large fillet grid.

Current allocation stacks localize growth to the welding spatial hash in
truck-meshalgo/src/filters/optimizing.rs: every 3D attribute is copied into 27 cells (2D: 9),
then neighbors are searched again. One-cell storage with neighboring-cell queries is the
smallest proposed correction. Preserve strict tolerance, transitive components, component
traversal/summation and representative choice; traversal changes can otherwise alter ties.
The one-cell storage fix is implemented; 25/125 cell queries and sorted discoveries
preserve the exact legacy candidate range, traversal, sums, ties and representatives.
The parent resource test fails (27 memberships per 3D attribute); 88 tests, 21 doctests,
six F9 tests, 2,048 randomized equivalence cases, Clippy and formatting pass. Uninstrumented
G2 now completes within the same 90 s / 3 GiB address-space / 1.5 GiB sampled RSS limits,
with closed mesh, unchanged input, valid STEP and unchanged independent material evidence.
Child HWM is 1,031,272 KiB. Native comparison remains failed. Independent welding review
approved the bounded delta with no blocking defect: candidate union, discovery order,
ordered components and exact representatives preserve legacy release behavior, including
wrapping bins. Stable combined validation and serial benchmarks remain acceptance gates.
No sampling/tolerance reduction is
justified; most of G2's dense face grid lies inside its trims. Evidence:
target/g2-mesh-resource-20261103/REPORT.md and its identity/geometry/allocation records.
Evidence: target/g2-welding-memory-20261103/results.json. No latency claim; sorting and
extra cell lookups still need neighboring performance measurements.



Termination comparison remains failed (.00344029 / .00210944). The 28 wrong-side
Truck-STEP classifications at .0001 offset are independently confirmed inward material
points (stations 68–95). Changing only the parabolic support's knot range to a 10x,
100x or 1000x chart removes all 28 classifications without changing spatial geometry,
edge/face tolerances, or classifier tolerance. Raw termination residuals remain tiny;
exported spatial-curve deviation stays inside the existing fitting tolerance. Imported
pcurves have larger same-parameter deviations, mainly tangential; their exact role in
classification remains unresolved. This is local chart-sensitive OCCT import/classifier
evidence, not demonstrated bad raw F9 material and not a comparison pass. Do not alter
kernel geometry or silently replace the original failed artifact with chart controls.
Evidence: target/f9-material-attribution-20261103/findings.txt, witnesses.json and provenance.json.

Independent review approves bounded raw F9 geometry/admission and the local chart
attribution; no blocking construction defect found. This is not imported-pcurve accuracy
or native comparison approval. The durable mesh-free classifier regression now lives in
audits/fillet-chamfer/oracles/f9_classifier.py and .cpp, with archived original/source STEP
fixtures. It checks all 198 independently derived sides on original and 10x charts at
unchanged tolerances. Current run exits 1, preserving 28 original failures and zero chart
control/source failures. Require both charts to pass before closing that classifier issue.
Evidence: target/f9-classifier-regression-20261103/results.json. No kernel geometry changed.

STEP helper coverage confirmed: step-test is a default truck-shapeops feature; ordinary
recorded Cargo test runs included it. Future combined validation must preserve default
features or explicitly enable step-test. Mesher review, bounded G2 resource investigation,
and P9 numerical-budget review continue independently. Stable combined validation
and serialized measurements remain pending; all F9 comparison failures stay recorded.



Other pending Nov3 integration evidence:

- **A1:** convex two-edge bevel-plane intersection fixes all three captures. Independent
  review approved; analytic wedge-volume parent failure and unchanged tolerances are in
  `target/a1-two-edge-20261103/`. Combined A1/H1 validation completed with 543 passes,
  one skip, seven doctests, 77 replay tests, Clippy and smoke. Full chamfer replay was
  486 operations / 358 constructions / 336 limited passes with no losses.
  `target/a1-h1-combined-20261103/validation-results.json` retains source/binary identity.
- **A2 triangles:** after A1, four additional DD/DA A7/A9 gains bring the recorded replay
  to 362 constructions / 340 limited passes. Seventeen curved-corner failures remain.
  Preserve A6 unequal-axis rejection, elliptic rejection, A7 and all 30 C2 successes.
  Equal A6/A8/A9 four-contact planar-support patches remain distinct from elliptic curved
  four-boundary patches and the DD/DA A6/A8 contact-order family. The A6 investigation is
  complete: native pivot selection depends on incident selection order; swapping it changes
  local material despite equal volume. Six fresh native permutations confirm this behavior.
  The current API promises selection-order independence, so no geometry-only invariant rule
  can match both choices. User chose an EXPLICIT PREFERRED EDGE for tied corners. Preserve
  selection-order independence; absent preference retains rejection. API/call-site design
  and implementation remain pending after this production freeze. Evidence:
  `target/a6-quad-contract-20261103/contract.txt`, material witnesses and policy probe.
- **H1:** bounded wall-first cubic construction without basis extension is implemented
  and reviewed. Original supports are retained; tests include seam incidence/opposite uses,
  fitted-interior material samples, right-root ordering and profile rejection controls.
  Independent flux volume is 367.45551955681 (source 372). Refined corner/canal witnesses
  support the tested rolling-ball material; native comparison remains failed. The older
  producer hash remains unresolved, but the follow-up's fresh artifacts match their producer.
  Evidence: `target/h1-implementation-20261103/`, `target/h1-local-witness-20261103/`,
  `target/h1-review-followups-20261103/`. No global root-completeness or containment claim.
- **C1:** all eight captured failures still preserve input. No new capture is admitted.
  Durable `c1_contact_isolation.py` and seven procedural oracle tests isolate fixed-station
  contacts using Bézier hulls, Jacobian/Krawczyk tests and rational Sturm station bounds.
  This is FP-margin evidence, not formal continuous certification. Necessary guide enclosure
  for equal 002 is approximately [-0.408749474535, 3.544880168161]; candidate retained
  interval [-0.018060160016, 1.023558799308] exceeds the wall chart at 0.995127641590.
  Original-support continuation, not prior-bevel substitution, is the candidate contract.
  The prior bevel remains a trimming face; an interior clip point cannot supply the same
  equal-distance contact. Existing 003 vertical-offset formulas are not exact for its stored
  degree-seven guide. Full trim membership, branch completeness, material containment and
  remote exclusion remain unresolved.
- **C1 repair dependency:** canonical spatial seam STEP #1326/#1330 is shared by bevel
  #1321 and cap #1833 with opposite uses. Proposed meeting is 9.93784e-8 from the original
  spatial curve; exact support intersection requires bevel u≈0.402461788894 beyond both
  trim and basis endpoint 0.402459605075. This is not disconnected-BREP proof. Replacing
  the preserved spatial edge with the cap pcurve or extending/reconstructing the inherited
  seam needs an explicit repair contract (stream E), not a tolerance change. Keep rejection.
  Evidence: `target/c1-next-contract-20261103/`, `target/c1-contact-isolation-20261103/`,
  `target/c1-retained-interval-20261103/`, `target/c1-trim-authority-20261103/`.

Coordinator's next integration gates:

1. Independent G2 welding review is complete with bounded approval. Retain the approved
   F9 mesh behavior and existing physical checks. Subsequent performance corrections also
   received bounded independent approval: checked insertion preserves Spade crossing and
   failure-before-mutation semantics; lazy enumeration/completed-component draining preserves
   legacy ordering. No blocking defect found; final integration passed.
2. Combined source-stable validation passed: 782 tests, 34 doctests, 77 replay tests,
   two expected skips, build/Clippy/smoke/format/diff checks. All 4,440 source/test/fixture
   hashes stayed unchanged with `step-test` enabled. All 486 chamfers completed at
   362 constructions / 340 limited passes, no lost checks; all 180 changed mesh volumes
   passed unchanged geometric allowances. Fresh G2 stayed inside the resource limits.
   P9 passes; G2/G6/H1/F9 comparisons and 28 F9 classifier failures remain unresolved.
   Evidence: target/nov3-combined-validation-20261103/REPORT.txt and results.json.
3. Serial measurements completed, preserving all 4,440 validation hashes. Nine matched
   cases meet measured native latency; H1 still fails geometry comparison and F9 is excluded
   for exact native-QA failure. Full evidence: target/nov3-final-performance-20261103/REPORT.txt.
   Confirmed plate regressions were isolated with identical dependencies to boundary
   constraint insertion under the division metric. Removing duplicate constraint traversal
   resolves them without changing metric or physical checks. Final plate10/30 medians
   1.247/4.125 ms versus isolated parent 1.276/4.199 ms (120 samples per side).
   Lazy key enumeration and completed-component handling preserve legacy welding order;
   dense/separated/transitive controls now beat legacy and preserve serialized results.
   Evidence: target/nov3-mesh-causal-20261103/REPORT.txt and results.json. Scoped 88 tests,
   21 docs, six F9 tests, Clippy/format/smoke pass; actual F9/G2 replays meet unchanged
   limits, G2 material witness byte-identical. Independent review approved correctness,
   causal isolation and recomputed timings; final integration passed. The archived
   2,048-case override predates this final delta; its final suite passes the unchanged
   equivalence test. Benchmark hashes cover the last result per process, and desktop
   contention remains disclosed. These are evidence limits, not demonstrated failures.
   Fresh final integration: 562 shapeops tests and 77 replay tests pass, one existing skip;
   all 491 replay records and 367 STEP DATA sections exactly match the combined baseline.
   All 4,441 source files unchanged; G2 meets unchanged resource limits and material checks.
   Evidence: target/nov3-mesh-final-integration-20261104/REPORT.txt and results.json.
   No global never-slower claim. The integration/measurement freeze is now complete.
4. CHANGELOG records the reviewed Nov3 behavior; add the A6 breaking migration after its
   implementation. Existing selection-order independence remains required. No commit/push/PR without a new explicit
   user request. The user has now requested completion of current work, commit, and stop;
   current C2 implementation and I2 budget research are closed. Commit validated changes
   and stop; do not start further capability work or follow-on investigations.

Closeout of current work (user requested commit and stop):

- G4/A5 historical benchmark evidence review completed with bounded approval.
- Selected-rim guard and material timing gate review completed with bounded approval.
- Rim-topology reuse received independent correctness/evidence approval; broader performance
  acceptance remains withheld.
- Controlled rim-ablation investigation closed with independent bounded evidence approval.
  Retain reviewed A4 improvement; neighboring parity remains unproven, all unfavorable
  history preserved. No deterministic source defect established; no speculative correction
  or further identical timing repetitions assigned.
- A5 DA geometry/preference review completed with bounded approval. Coordinator corrected
  reentrant API wording only; executable source unchanged. Fresh post-correction benchmark
  preparation passed all 22 engine/case smoke/material gates and eleven integrity tests.
  Coordinator verified current campaign identity, both-engine smoke and complete material
  proof, and published material-approved.json. Serialized matched/neighbor measurements
  completed and independently approved for the bounded valid-native target. Neighbor
  parity remains unproven; no new deterministic production defect established.
- Remaining chamfer partition completed: five already-supported explicit-choice cases,
  four opposite curved-contact-order failures, three C2 modes and one C1 spline network.
  Equal-distance complex C2 is implemented and validated; DD/DA and unrelated families
  remain excluded. All current delegated work is closed.
- I2 ideal outer seam, curved remote termination and eight-face assembly derived.
  Independent ideal-contract review completed with bounded approval. Representation-budget
  research is closed at the user stop request: support displacement and proposed canal-fit
  bounds are recorded, but shared-trim derivatives, endpoints, stored-control and
  same-parameter budgets remain unproved. No I2 production admission or capture gain.
  Evidence: target/i2-representation-budget-20261104/REPORT.txt; all I2 processes joined.

Independent DA benchmark review found no blocking measurement defect. Every reported
statistic and 328 logs were verified, with 1,082 acceptance artifacts, 479 preparation
artifacts and 19 supplemental pins matching. The measured valid-native target is accepted
for this explicit chosen-shape case. A6/A1 DA unfavorable neighbor results remain retained;
no parity, regression allowance or universal speed guarantee follows from the predeclared
investigation threshold. No basis for another identical repeat campaign was established.

A5 DA measured median/P95 is 5.099084/5.397157 ms versus valid native
11.032289/11.299064 ms in the frozen campaign. Parent DA has no successful baseline.
Initial/reversed neighbor controls retain unfavorable A6 and A1 DA ratios; no parity or
causal regression conclusion follows. The campaign's >5% flag is only an investigation
threshold, not an authorized relaxation or equivalence test. Independent review must
check its preregistration, gates, timer boundaries, baseline comparability and all raw
series. No further repeats or production edits were performed. Evidence:
target/a5-da-acceptance-20261104-serialized-01/REPORT.txt and results.json.

A5 DA now constructs with four explicit input-bound corner preferences. The new signed
convex family and unique regular noncollapsed reentrant pivot have independent material,
trim and footprint checks; default DD/DA still reject. Annotated DA exact QA and 10,293
whole-boundary finite material witnesses pass, sampled distance <=1.76e-12. Final validation:
615 tests (one skip), seven doctests, 77 replay tests, Clippy/format/smoke; default full486
remains366/344 with no lost checks. Both new tests fail frozen incoming production, not
compilation. Existing A6, G4/G8 and rim-reuse controls are preserved. Bounded native-target performance is now reviewed and accepted; neighboring parity remains
unproven. Independent review approved the pivot semantics, regularity,
trim/footprint preservation and parent proof; all owned/source/executable hashes matched.
Independent oracle and both exported material checks passed. The nonblocking rustdoc
correction now explicitly excludes the regular noncollapsed reentrant family from default
triangle behavior. Non-rustdoc lines are byte-identical; no geometry retest needed for that
wording fix. Evidence: target/a5-da-review-doc-20261104/validation.json and
target/a5-da-implementation-20261104/REPORT.txt and production.patch.

Independent controlled-ablation review found no blocking adapter/evidence defect. Exact
variant/build/runtime/input/material gates, all raw series and integrity controls were
verified. It supports closing this bounded causal investigation: separate build/helper
paths and differing same-source executables prevent isolation of a machine-code effect;
neighboring parity is not proved. A4 correctness and measured gain are retained. Further
causal attribution would require a genuinely different measurement capability and bounded
question, not repeated unchanged campaigns. G2/G6/H1/F9 geometry failures and F9 invalid
native-QA timing exclusion remain unchanged. No new code edit or performance guarantee.

The controlled campaign built incoming, reviewed candidate and a second same-source candidate
from fresh targets. Other source entries, helper/input/runtime identities and compiler/profile
settings matched; separate target paths and differing static archives/executables remain
confounders, explicitly recorded. Complete fresh per-producer smoke/material gates passed.
Both predefined interleaved series are retained. A4's gain persists; A6 remains unfavorable,
G8 reverses direction, and DA differs substantially between same-source candidate builds.
DD remains uncertain. This does not isolate a deterministic rim-helper source regression or
establish parity. The earlier extra A6 mallocs did not recur; newly observed connectivity-vector
reallocations are distinct events, with pointer/hash traversal effects plausible but not proved.
No permanent edit or new full-suite claim. All processes joined, 7dfb9b56 restored. Independent
review must assess the scoped adapter, actual intervention limits and bounded disposition before
acceptance. Evidence: target/controlled-rim-ablation-20261104-075701/REPORT.txt.

Nine bounded exact-binary phase profiles completed without code edits or new timings.
A6's explicit fully selected preference path skips tangent admission; two allocation-site
changes remain unattributed. G8 allocation/subdivision counts match across producers and
never enter chamfer; A1 DA admission allocates less, with miter/finish counts unchanged.
All nine fresh shape checks pass. Compiler/profile/runtime provenance was checked, but
static archive differences and unlocked host frequency do not establish a latency cause.
No speculative corrective rewrite is justified. Next controlled intervention changes only
the owned rim delta under matched fresh build conditions and traces the A6 allocation sites;
prior unfavorable measurements remain retained. Evidence:
target/remaining-causal-20261104-074951/REPORT.txt.

Independent rim-topology review found no blocking correctness defect or essential missing
proof. Archived/current candidate matched the pinned 7dfb9b56 source. The reviewer confirmed
topology predicate equivalence, all replay JSON/STEP parity, material approval identities
and both predefined timing series. All 4,757 campaign, 5,870 predecessor and 2,478 original
artifact hashes matched. A1 material approval remains finite/local. The remaining A6/G8/DA
signals do not by themselves causally implicate this helper; their separate attribution task
continues. No new builds or timings were run during review.

Selected-rim topology reuse now has 613 passing tests (one skip), seven doctests, 77
replay tests, Clippy/format/smoke, and full486366/344. All incoming JSON records and all
366 successful STEP DATA sections match exactly. Fresh complete both-engine material
approval passed, including rerun A1 checks; historical material-adapter outputs were not
used to authorize the new campaign. Only chamfer_miter.rs changed, including equivalence
coverage of selection, orientation, shared identity and coincident distinct vertices.
A4 is faster against both original and incoming in the two predefined series. Broader
acceptance remains withheld: annotated A6 versus its earlier producer and G8 versus incoming
remain unfavorable; A1 DA is unfavorable versus incoming, while original DA and DD ratios
reverse direction. These are distinct evidence boundaries, not proof the rim helper caused
unrelated fillet latency. No additional unchanged timing series is authorized as a substitute
for causal investigation. Evidence: target/rim-topology-20261104-071836/REPORT.txt and owned.diff.

Combined G4/A5 correctness accepted on verified frozen evidence: 611 tests, 7docs/77replay,
full486366/344 no lostchecks; exact reviewedG4 present. Fresh app261 passes. Reused checks
are source/dependency/command-bound, not stale pre-refinement G4 runs. Matched G4/A5 meet
valid-native latency target, but performance acceptance remains withheld: A4equal andA1DD
small positive shifts persist in both predefined neighboring campaigns. Profiles show
changed A5 planning calls but do not apportion cost. Correction and freshvalidation assigned.
Evidence: target/g4-a5-final-validation-performance-20261104/REPORT.txt.
Independent review recomputed raw replay/statistics and verified provenance/gates; no
additional blocker to the historical withheld-performance conclusion. Later cfg(test)
regression and evidence-driver correction are separate from publication-time identity.
Reusable g4_a5 timing CLI now enforces material-approved.json (or --material-approval):
identity must match the campaign, both engines of every timed case need positive witness
counts and empty mismatches, and evidence hashes must include the corresponding smoke STEP.
The approval digest and all evidence are rechecked before summary publication. Ten integrity
tests pass; identical tests against the incoming script expose twelve failed subtests.
No geometry runs or timings were needed; historical campaigns remain unchanged. Historical
partial approvals cannot authorize a new run. Evidence:
target/g4-a5-material-gate-20261104/validation.json.

The necessary selected-conic-rim guard is implemented before unrelated planar planning.
Validation: 612 tests, seven doctests, 77 replay tests, Clippy/format/smoke and unchanged
486/366/344 replay results. A4 still regresses against the original baseline; A1 DD's
improvement versus incoming is inconclusive. A4 does not traverse the removed planar work,
so that mechanism cannot explain its residual. Performance acceptance remains withheld.
Original evidence and all geometry gates remain preserved. Independent review approved
the guard as a necessary-condition rejection and approved the material timing gate; no
blocking defect found. The reviewer reproduced all ten integrity-test passes and the
incoming script failures, and verified all 2,478 historical and 5,870 campaign hashes.
The material gate authenticates supplied results/evidence, not witness sufficiency.
Historical causal A4 investigation was subsequently closed without a speculative rewrite. Evidence:
target/g4-a5-neighbor-performance-20261104/REPORT.txt and candidate.diff.

A1 finite local material adapter now covers the two-selected-edge corner, strips and
untouched neighbors using input-derived planar trims and labelled setbacks, not output
geometry or native classification as expected material. Each case/engine has 369 probes;
historical equal/DD/DA both-engine checks and causal DD checks pass. Independent analytic
checks and changed-output/expected-state negative controls pass. It does not cover every
A1 feature or certify the whole solid. The fresh campaign must rerun check.py with explicit
smoke paths and merge identity/proof/results plus the output report hash into its approval.
No production/build/timing changes. Evidence: target/a1-neighbor-material-20261104/REPORT.txt.

The subsequent causal-path investigation made no production/test changes and no new
latency claims. It localized one additional 40-byte Face::edge_iter allocation to the rim
guard; that cannot explain the pre-guard regression. Miter allocation counts match the
original baseline; emitted code layout differs, with no established latency attribution.
Fresh build, both-engine A4/A1DD smoke, one focused test and ten harness tests pass.
No material approval or new timings were generated. All processes joined. Evidence:
target/a4-causal-path-20261104-070956/REPORT.txt. Next intervention targets demonstrably
repeated topology queries in tangent_neighborhood, not speculative inline/layout changes;
any measured gain must be distinguished from proof of the earlier regression's cause.

A5 review follow-ups closed by coordinator after freeze: direct Rust collapse-policy test
checks .29/.31 success plus exact/near .3 rejection and input preservation; passes. The
initial fixture swapped support-distance labels, corrected with failed log retained.
Production prefix unchanged. Scoped Clippy (known warnings), nightly format/diff pass.
Material driver now exits1 on mismatches; mocked matching/failing classifier runs verify
exit0/1. Four historical material rows independently checked empty; no old outputs/manifests
rewritten. Original driver archived before correction. Evidence:
target/a5-review-followups-20261104/validation.json and driver-check.json.

A5 equal and explicitly preferred DD are implemented on the frozen recorded snapshot.
611 tests (one skip), seven doctests, 77 replay tests, Clippy/format/smoke pass. Whole
input-derived flux/material includes all 18 faces/19 wires and hole; exact output QA and
sampled comparison pass. Full486 now366/344, only unannotatedA5equal gain and no lostchecks;
annotatedDD passes separately. UnannotatedDD, unsupportedtrianglepreferences andDA reject
immutably. A6 guards and oldreentrant8facehistory preserved. Independent review gives bounded
approval: all4075artifacts and1448priorartifacts verified, input-derived wholeoracle and
rawreplay counts/material states independently checked. No demonstratedcorrectnessblocker.
Combined acceptance/performance pending. Evidence: target/a5-assembly-20261104/REPORT.txt/results.json.

A1 DA attribution completed without a justified corrective production delta. Same
successful miter path and main-phase allocation/construction counts across exact binaries;
no fallback regression. Candidate4 has three extra small dispatch allocations, but their
latency effect is not established. Codegen/layout leads remain unproven. One predefined
balanced 12-round/120-sample-per-binary interleaved experiment did not reproduce a positive
aggregate signal; historical unfavorable series remain retained. This does NOT establish
equivalence or resolve the historical slowdown causally. No speculative inlining/rewrite
or repeat-until-pass timing is warranted. Residual performance uncertainty remains open
while independent A5/G4 work proceeds. Evidence: target/a1-da-latency-attribution-20261104/REPORT.txt.

Single-collection reuse is implemented in chamfer_miter.rs; planar.rs changes only the
coordinator-authorized pub(super) validator visibility. Gates, diagnostics, tolerances
and face-history behavior remain unchanged. Final validation passes 591 tests (one skip),
seven docs, 77 replay tests, Clippy, smoke, formatting and materials. Full 486 remains
365 constructions / 343 limited passes with every check unchanged.
Original A1 equal/DD and A7 regression mechanism is removed; A2/A9 increases did not
reproduce in longer confirmation. A1 DA remains unresolved: final 32-round point estimate
+2.42%, exploratory interval -0.32% to +4.29%, with earlier positive series retained.
Do not infer equivalence from an interval crossing zero. The completed focused attribution below does not justify another production fix. No global
performance acceptance yet. Evidence: target/a4-admission-performance-20261104/REPORT.txt.
Independent candidate4 review found no concrete correctness blocker: descriptor lifecycle,
validator/error ordering, explicit-preference routing and unchanged construction verified.
All 1,933 original-evidence hashes match; raw replay confirms zero changed checks.
The report authorization wording is corrected to coordinator authorization under existing
user instructions; original report bytes are retained beside it. Performance remains open
for A1 DA; review does not treat an interval crossing parity as equivalence.

Initial A4/G8 timing campaign completed; performance acceptance remains open.
Fresh matched comparison/material/exact-native-QA/identity gates pass. Forty samples
per engine/case show new A4 and G8 paths below OCCT latency, but reversed-order neighbor
confirmation finds A1 equal/DD about 9–10% slower and A7 about 3–4% slower than the
verified incoming baseline. GDB profiles show two extra collect_neighborhood calls
in A4 admission; debugger results establish call counts, not apportioned latency.
A9/A2 smaller signals require confirmation/attribution. Fix and remeasure before closing.
Evidence: target/a4-g8-performance-20261104/REPORT.txt and retained raw records.
No production changes occurred during that campaign. Native failed statuses remain failed.
Independent review verified all 732 listed artifact hashes, incoming-baseline isolation,
timer boundaries, both-engine gates, raw statistics and reversed-order confirmation.
Material counts clarified: 42 witnesses per A4 mode, 126 across three modes; 770 G8
witnesses per engine. No additional measurement blocker. The later chamfer_miter fix
is outside this campaign approval and requires fresh correctness/performance evidence.

G8 is implemented with a separate cubic admission/disk-containment certificate and two
exact cylinders. Nine focused tests and 770 raw/exported material witnesses pass; fresh
G8 .001 sampled comparison passes (maximum 5.433e-6). Docs, 77 replay tests, Clippy and
smoke pass. Direct incoming-source positive fixtures reject before the fix; frozen archive
and original captures remain unchanged. Evidence: target/g8-implementation-20261104/REPORT.txt.
Independent review gave bounded approval: interval uncertainty/double-root algebra,
admission, trim assembly, evidence hashes and independent oracle checked; no concrete
blocker. Coordinator added requested direct uncertain/wrong-bound certificate controls,
adjacent-float radius admission controls and complete face-history correspondence checks.
Standalone extracted certificate and actual crate unit test pass; all nine final focused
integration tests pass, including radius admission and full face-history correspondence.
Scoped release Clippy, nightly formatting and diff checks pass. G8 non-test bytes exactly
match the reviewed hash. Logs and final hashes: target/g8-implementation-20261104/
review-followup-validation.json. The initial history-test compile error (Surface lacks
PartialEq) was corrected using serialized oriented supports; its failed log is retained.
The later settled A4 campaign passes 591 tests with one existing skip. Its partial-only
unchanged-face reuse restores the reentrant chamfer modified-face count to eight without
changing assertions. Production stayed frozen; the two authorized G8 test-only changes
are separately hash-bound and pass final focused checks. Initial timings are complete;
neighbor regression fixes and final measurements remain pending.
G8/G4/P9 and G2/G6/H1/F9 controls retain their recorded respective pass/rejection/failure
statuses; no generic cubic or global formal accuracy claim.

A4 tangent-cylinder partial contours have bounded independent geometry approval;
performance acceptance is held for the measured neighboring regressions.
Only the three A4 equal/DD/DA captures gain: full 486 replay now has 365 constructions
and 343 limited passes, with no lost checks against the incoming 362/340 baseline.
Shared cone/plane generators retain original loops and untouched supports; analytic
local trim exclusions conservatively reject collisions. A5 additional corners remain open.
Fourteen focused tests, 591 combined tests (one skip), seven doctests, 77 replay tests,
Clippy, formatting and smoke pass. Independent A4 and annotated A6 material checks pass.
Evidence: target/a4-tangent-implementation-20261104/REPORT.txt and validation-results.json.
Independent read-only review verified owned/parent/material hashes, admission, shared
generators, conservative remote bounds, terminal rejection and restored reentrant history.
Raw replay recomputation confirms only three A4 gains and no lost checks; no reproduced
or source-proven blocker. Approval excludes A5, performance and global certification.
Full-ellipse bounds may conservatively reject safe tight geometry.

A6 explicit-preference implementation, review, integration and measured performance are complete:

Kernel implements the specified API, pivot validation/assembly and explicit replay
annotations. Recorded validation: 565 release tests (one skip), ten focused STEP tests,
77 replay tests, seven doctests, Clippy/format/benchmark smoke. All 486 unannotated
captures remain 362 constructions / 340 limited passes with no incoming-result changes.
Explicitly annotated A6/A8/A9 each pass sampled comparison and 138 material checks.
Frozen parent rejects the new construction; all 4,367 parent source hashes preserved.
Evidence: target/a6-preference-kernel-20261104/validation-results.json and owned-files.json.
These are explicit chosen-shape results, not new default unannotated capture passes.
Kernel review approved the bounded construction, API/pivot validation, independent material
evidence and frozen-parent identity. Coordinator closed its essential coverage request:
large-translation valid/perturbed d=2h pairs at offsets 0, 1e3, 1e6 and 1e12. Invalid pairs
must reject immutably; valid successes must preserve local contacts, signed material and
whole-solid checks, with conservative rejection allowed beyond numerical limits.
All 11 curved-network tests, focused release Clippy, formatting and diff checks pass.
Only the test file changed; production/geometry/tolerances stayed unchanged. Logs:
target/a6-preference-kernel-20261104/large-translation{,-full,-clippy}.log.
No remaining demonstrated kernel review blocker. New-path timings and final app integration
remain gates. The combined validation's two app face-count failures are resolved as obsolete
expectations from intentional Oct 25 commit 2cf44b16, not A6. Frozen d89c7368 reproduces
identical fixture contacts/volumes: three bevels plus a separate planar triangle, equal
volume 999.405333333333 rather than the old apex 999.406. Only blends.rs tests changed;
unchanged tolerances now check exact topology, triangle vertices/material orientation,
analytic volume and distinct reference identity through upstream edits. Shared Blend slot 0
matches the documented naming contract. All 261 core units/blends/face-blends pass without
fail-fast; scoped Clippy passes. Evidence:
PiezaCAD/target/chamfer-corner-integration-20261104/REPORT.txt. Final coordinator inspection
confirmed strengthened assertions and frozen-parent attribution; no production fix needed.
The explicit-control coverage gap is closed by the new headless test in
PiezaCAD/cad-app/tests/control.rs. Real named preferences reach the kernel on an unsupported
box and return corner_preference_network without fallback; its empty-preference chamfer
succeeds. Persistence, legacy defaults, editor pending/completed rows, reorder/deletion,
Apply guards, undo/redo, save/load/rebuild, malformed/duplicate/nonincident/unselected/stale
refs and original geometry/names are checked. This does not establish accepted A6 app
geometry or parent-binary failure. New runtime test, five entity-selection unit tests,
three field controls, two blend controls and all-target app/core check pass. Evidence:
PiezaCAD/target/chamfer-preferences-runtime*.log and runtime-provenance.sha256. Test-only
change; no runtime-test processes remain. Final validation closure passed: fresh 261 core
tests, 81 app units, headless preference control, all-target check and Clippy; all three
annotated replays and smoke-output material oracles pass. Production/tests/docs and external
dependencies stayed frozen. Evidence: target/a6-final-validation-benchmark-20261104/REPORT.txt.
Matched explicit-shape medians/P95 ms (Truck vs OCCT), 40 samples each:
A6 1.780/1.839 vs 4.104/4.257; A8 1.752/1.843 vs 4.116/4.192;
A9 1.800/2.262 vs 4.094/4.191. Eight neighbors show no reproducible median regression;
initial A2/A1 increases reversed in bounded confirmation. Material witnesses run on smoke
outputs; each iteration checks consistency/repeatability/immutability and exact native QA.
No accepted captured A6 UI claim or universal latency guarantee. Independent review approved
the retained campaign and found a reusable-harness gate defect: a failed Truck comparison
could permit timing. Coordinator corrected both A6 and Nov3 scripts to require both engines'
comparison passes. Identical regression fails both incoming scripts; all seven integrity
tests now pass for each. Verified all three A6 and eight Nov3 timed cases have both comparison
passes. H1 retains its explicitly failed geometry comparison and diagnostic construction
timing; it is not a geometry-matched performance pass, and the corrected gate blocks future
matched timing. Historical records/caveats remain intact; A6 retained timings are unaffected.
Corrected the retained report generator's smoke-equality overclaim to within-process
repeatability; published REPORT.txt already had correct wording and was not regenerated.
Original generator/scripts preserved with before/after hashes; old manifests remain original
campaign evidence, not rewritten. Evidence: target/a6-benchmark-review-fixes-20261104/validation.json.
Future campaigns need fresh build/smoke identities after script changes. No remaining
demonstrated A6 review/integration/performance blocker; all owned processes finished.

PiezaCAD implementation has completed review and the focused row fix: default-empty persisted references, strict
resolution/body partitioning, simultaneous adapter routing, explicit editor rows and
control support. `cargo check -p cad-core -p cad-app --all-targets`, six preference unit
tests, six face-blend tests and editor row/reordering test pass; final selected-edge
tightening passed affected core tests. Logs: PiezaCAD/target/chamfer-preferences-*.log.
App review found one row-dispatch blocker; coordinator fixed it by deferring completed-row
removal until pending-row edit dispatch finishes. The new regression fails before the fix
and now passes for both corner/edge pending selections and Clear/Remove operations; the
incomplete-pair Apply guard remains required. All five entity-selection tests pass and
diff checks pass. Logs: PiezaCAD/target/chamfer-preferences-row-{before,after}.log.
Strict resolution, ownership, atomic body replacement, persistence and no-fallback routing
otherwise passed review. No remaining demonstrated app review blocker.
No GUI validation or end-to-end accepted A6 feature claim yet; current per-feature sizing
limitation remains. Combined validation now checks both completed streams.

- Add `ChamferCornerPreference { vertex: VertexID, edge: EdgeID }` and an unordered
  preference slice to `try_chamfer_solid_edges_with_distances`, before `tol`. One selected
  admissible edge per tied vertex permits independent choices at opposite caps. Existing
  callers pass `&[]`; uniform/wire APIs retain their current purposes.
- Reject missing, duplicate, conflicting, stale, unused, unselected/nonincident or axis-pivot
  preferences. Preference-bearing requests must bypass uniform shortcuts and split/wire
  fallback: either construct the explicitly chosen simultaneous network or return an error.
  Validate before assembly; preserve input and selection/preference-order independence.
- Admit only the bounded fully selected one-cylinder planar network with local-frame
  setbacks `d = 2h > 0`. Select the pivot by original edge identity; update all four patch
  neighbors, including original radial support B–C, shared topology and retained trims.
  Other partial, elliptic and DD/DA families remain excluded. Preserve empty-preference
  A6 rejection regression; add all four independent cap-choice combinations and signed
  material witnesses, not just equal-volume checks.
- PiezaCAD persists `Ref<VertexName>` and `Ref<EdgeName>` pairs in a default-empty Chamfer
  field, never transient kernel IDs. Resolve body ownership and incidence unambiguously;
  strict preference resolution must reject tied name candidates. Propagate through feature
  dependencies, build/kernel adapter, explicit editor rows, control API and save/load.
  Nonempty preferences prohibit grouped/sequential adapter fallback. Current UI per-feature
  distance pairs cannot author the original captures' per-edge sizing; do not claim otherwise.
- Replay preferences must be explicit identity-bound geometric descriptors, uniquely resolved
  after import; archived captures stay unchanged. Unannotated requests retain rejection.
  Compare annotated output as the deliberately selected shape, not native default behavior.
- Migrate kernel examples/tests/benchmarks and all app callers; add Breaking CHANGELOG/API
  and control documentation. Rebuild benchmark identities after migration. Parent proof must
  fail construction/material assertions with test-only API adaptation, not fail compilation.

Design and implementation evidence above remain bounded to the annotated family.
Underlying geometry evidence: `target/a6-quad-contract-20261103/contract.txt`.


Shared-checkout ownership is coordinated; no stale scratch-worktree sources may overwrite
main. T3 worktree creation previously failed with "Unable to resolve the parent checkout
commit"; do not create child checkouts in shell. The frozen `d89c7368` source archive is
`target/h1-parent-d89c7368-20261103/source`, with 4,367 source hashes and archive provenance.
Parent production remains immutable; only identical regression fixtures may be added.
Coordinator owns TASKS.md and integration. Timings remain serial after concurrent work.

### Previous completed batch — 2026-10-27

Completed batch: bounded quarter-cylinder and fully selected cylindrical chamfer networks,
bounded parabolic chamfer continuation, quartic G6 wall-first fillets, and shared derivative
optimizations. Tests, geometry/material oracles and CHANGELOG describe the accepted scopes.
The latest full chamfer replay covers **486 operations: 358 constructions, 333 limited
comparison passes**. A7 is the new gain; no lost checks. All 30 C2 passes remain. The eight
C1 failures and 21 remaining A2 failures preserve inputs. No fresh full native recapture.

Final broad shapeops run: 538 passes, one existing skip; seven doctests and 77 replay tests.
The subsequent guard-only regression addition passes all six network tests and Clippy.
Geometry optimization validation: 173 passes, one existing skip and the known
pcurve_derivation precision failure; 139 geometry doctests. Scoped formatting, release
Clippy and benchmark smoke pass. Independent reviews approved bounded geometry and
performance changes. A2 review's guard-coverage gap was closed by an axis-edge-only
unequal setback test: bypassing only the symmetric-planar guard constructs the rejected
fixture; restoring it passes scaled/reordered controls. No tolerance or sampling changes.

Serial matched benchmarks, CPU0 / one worker / 40 samples per configuration:
- G6: 25.54 ms median versus OCCT 30.81 ms (previous Truck 46.67 ms).
- G2: 18.12 ms versus OCCT 29.05 ms (previous Truck 25.13 ms).
- A7 network: 0.206472 ms median / 0.271100 ms P95 versus OCCT
  2.984592 / 3.174877 ms. Fresh sampled distance at most 8.52e-15.
- Old bounded A2 remains faster than OCCT; no material neighboring regression shown.
- C1 standalone approximately 9 ms; a matched OCCT comparator is still unvalidated.
These measurements do not establish universal performance. No new persistent caches;
allocation totals were not measured in this batch.

G2 and G6 native comparisons **remain failed**. Independent local sphere/canal material
witnesses support Truck's rolling-ball construction; they are not global uniqueness,
containment or nonintersection proofs. G6 corner distance ~0.185900501 and canal distance
~0.001881877 remain recorded. Fresh native tracing observes three-boundary Coons filling,
but does not establish original-capture binary provenance. Reference-exception accounting
remains a user decision; never count these as native comparison passes.

Retained evidence (under target; do not delete before reproducing or archiving):
- g6-performance-20261027/RESULTS.txt, baseline archives and final/validation logs.
- g6-combined-integration-20261027/validation-results.json.
- a2-assembly-20261027-092345/validation-results.json, implementation.patch,
  guard-bypassed.log, guard-restored.log and guard-clippy.log.
- a7-network-benchmarks-20261027/RESULTS.txt and raw samples/provenance.
- c1-integration-validation-20261027/validation-results.json.
Durable G6 witness/oracle sources are in audits/fillet-chamfer/oracles/.

Remaining priorities: A2 two-edge/unequal planar networks and elliptic patches; broader
C1 continuation; H1 cubic fillets; matched C1 OCCT timing. Preserve the bounded guards
rather than broadening whole-shell replacement. Existing scratch worktrees and their
uncommitted copies are retained for evidence, not active work; main contains the integrated
batch. Do not overwrite main from those older copies.

## Streams

Unequal C2 review corrections (2026-11-05; admission fixes now approved):
- Map public shell-face-order distances explicitly: for incident indices a,b, cap setback
  u=d[usize::from(cap==max(a,b))], axial v is the other entry. data.sides order is
  oriented-edge order, not the public distance indexing. Test reversed stored edges and
  shell reordering with correctly remapped unequal distances.
- DA bottom requires the nondegenerate parabolic Q seam (equal Ro/H slopes). It must
  succeed, with nearby slopes on both sides, cylinder continuation and generator limits;
  endpoint matching alone does not establish the positive-x/monotone-axial branch.
- Combine both cap offset laws instead of selecting by the axial halfway point. The
  coordinator's target/c2-unequal-corrections-20261105/classifier.py fixes this independent
  classifier, preserves all 9,208 recorded captured classifications and checks the legal
  asymmetric L=1.5 fixture plus its reflection. Prior artifacts remain hash-identical.
- Budget u/v sensitivity, axial coordinate error and near-parabolic conic construction;
  reject uncertain conditioning rather than inheriting equal-case bounds unchanged.
  Derive the numerical volume allowance for each admitted scale/profile.
- Constant mismatch h_i-h_o yields constant radius sum and an affine intersection plane,
  hence may reduce to a conic. General independently varying slopes can produce quartics.
  Incompatible tangent-chain pairs remain excluded; no new policy is authorized.
No unequal C2 construction gain yet. P1 other-shell and P2 plane-displacement corrections
are resolved by the reviewed five-file admission correction. Unequal contract review:
target/c2-unequal-review-20261105. Original equal-C2 review findings were delivered in
thread messages; no target/c2-independent-review-20261105 directory was produced.

Bounded equal-distance complex C2 is implemented in the private chamfer_extrusion
constructor with narrow dispatch. It preserves nine original supports and builds shared
tangent generators and exact plane/cone/cylinder trims, including the longer high-side
bevel's continuation on the retained cylinder. Independent volume/material, history,
topology, STEP, transforms and immutable rejection controls pass. The initial failing
procedural test and imported rejection remain preserved as parent evidence at
target/c2-implementation-20261104; final evidence is target/c2-constructor-results.json.
Only the seven-piece equal-distance analytic extrusion is admitted. DD/DA, elliptic
and spline families remain excluded. Numerical bounds use floating-point arithmetic;
sampled native agreement is not global accuracy certification. No C2 timing claim.

The other four A6/A8 DD/DA failures are contact-order obstructions, not solved preferences;
the elliptic procedural filler and C1 spline network are separate open families.

### A. Chamfer corners (12 unannotated failures; 5 support reviewed explicit corner preferences)

Owns `truck-shapeops/src/fillet/chamfer_miter.rs`, `chamfer_edges.rs`. Policy: follow OCCT's
corner construction (decided 2026-10-24). Reproduce OCCT behavior with
[probes/chamfer_corners.cpp](audits/fillet-chamfer/probes/chamfer_corners.cpp).

1. **Two-edge convex junctions:** implemented and reviewed in the pending Nov3 batch;
   all three A1 discrepancies now pass targeted/full chamfer replay. Preserve the independent
   wedge-volume regressions and concave behavior during combined validation.
2. **Corners on curved supports and arc edges** (conical bevels):
   `chamfer/{equal_dist,dist_dist,dist_angle}_complex/{A4..A9,C2}` and
   `chamfer/equal_dist_complex/C1`. Exclude passing equal_dist_complex/A7/001 and
   DD/DA A7/A9/001. Their final error `intersect_straight_blends` is the
   straight-edge fallback, not the first failure: record the first failing constructor per
   case before designing. Selections expand to 8–21 edges; partition them into local
   corners first. Lead: [triage §3](audits/fillet-chamfer/notes/triage-fillet-junctions-and-chamfers.txt).
   A4's three modes now pass with exact shared tangent generators and local trim
   exclusions; reviewed implementation and 365/343 full replay recorded above.
   Current A5 equal and explicitly preferred DD are implemented and reviewed (see
   target/a5-assembly-20261104/REPORT.txt); default DD and DA still reject. The following
   entries retain prior research chronology. New work targets bounded explicitly preferred
   DA using the existing API, preserving A4/A6 and unannotated rejection contracts.
   Initial contract: target/a5-contract-20261104/contract.txt. Verified retained-library
   procedural fixtures reject all three modes at partial_tangent_domain; first blocker is
   chosen>2 in tangent_neighborhood. One convex triple {21,22,37} has input-derived
   signed-section bilinear contacts with 54 native material states passing. DA violates
   every existing monotone pivot there. Do not simply relax chosen>2 or triangle gates.
   Whole-A5 corner-choice uniqueness and independent whole-solid volume remain unresolved;
   completed follow-up: target/a5-corner-choice-20261104/contract.txt. All ten triples
   partition into four orthogonal convex, two skew convex, three mixed reentrant and
   one fully reentrant. The 180 full-fixture native permutation builds showed no local
   material differences on the finite grids, but 24 isolated builds establish three DA
   outcomes for the same local geometry. Captured signed-section DA needs explicit pivot
   edge21, identified persistently rather than by array index; no new API policy decision.
   Independent whole rounded-prism section volume/material now covers opposing caps and
   remote holes. This is NOT a whole-A5 oracle. Six skew/reentrant local plans remain
   unverified, so no complete equal/DD/DA capture is admitted. Current research targets
   those plans and potential reuse of existing reentrant miter logic. A5 remains rejected;
   no relaxation of chosen>2, unequal_triangle_relations or A6 preference gates.
   Six skew/reentrant candidate patch types are now identified from input contacts:
   target/a5-reentrant-contract-20261104/contract.txt. Equal/DD use triangles at the
   fully reentrant and two skew-convex triples, signed bilinear sections at three mixed
   triples; DA fully-reentrant candidate is signed bilinear. All 324 local material
   witnesses agree; contact discrepancies below 6e-13. Existing planar reentrant branches
   do not reproduce those material contracts unchanged. Candidates were matched to native
   for reporting, NOT a production choice rule. Isolated-source choice, exhaustive local
   material partition and assembled footprint clearance remain open. Next experiment
   constructs exact 7-/3-sector oblique source corners, varies selection and remote extent,
   and checks a closed local material/flux oracle before any production admission.
   Seven-sector equal/DD isolated experiment is complete:
   target/a5-isolated-choice-20261104/contract.txt. All 24 exact-QA native outputs match
   the input-derived triangle rule, including all orders and two remote extents; closed
   partition, crossing, flux and clearance checks pass. Three-bevel is also independently
   valid but differs from native material. Coordinator adopts the fixed triangle for this
   bounded family under Stream A's existing follow-OCCT construction policy, not a claim
   of geometric uniqueness; no new transition-kind API is needed. Explicit preferred-edge
   policy remains for genuine signed-pivot ties. Mixed three-sector and skew-convex isolated
   contracts are now specified in target/a5-mixed-skew-contract-20261104/contract.txt:
   all 120 native results/20 sources pass exact QA, all six orders agree, and 28 closed
   local models pass partition/incidence/flux/clearance checks. Native material gate has
   six failures: the same inward DD{6,7,16} witness at L=2, one per order. L=1 and both
   independent crossing oracles give IN; L=2 native gives OUT despite bilinear geometry
   matching below 1e-13 and equal nearest-face distance. No all-native-pass claim.
   Follow-up isolates this comparator contradiction without modifying BREPs/tolerances;
   actual 18-face assembly and whole-A5 material oracle remain pending. All ten equal/DD
   plans are specified; DA remains explicit-choice work, no blanket admission.
   Comparator contradiction resolved locally: target/a5-classifier-attribution-20261104/REPORT.txt.
   Exact represented witness is IN. Native L2 uses a remote ray near-edge event at 7.14e-5
   within stored edge tolerance 1e-4, producing a false entering crossing. Independent
   bilinear/closed-boundary crossings confirm membership. All six original native failures
   remain failed; no BREP/tolerance change or all-native-pass claim. This permits bounded
   local geometry implementation with the comparator limitation explicit. Equal/DD actual
   18-face assembly, full material/flux oracle and parent-failing regressions now assigned;
   DA and unrelated preference/admission families remain outside scope.
   Initial implementation attempt stopped without production/test changes or a concrete
   blocker; rejection/provenance evidence retained in target/a5-implementation-20261104/.
   This is incomplete, not a geometry no-go. Reassigned actual certified local-plan and
   per-wire assembly implementation, reusing parent evidence rather than restarting research.
   Full assembled oracle and validation remain required; no A5 gain claimed.
   Implementation exposed a choice gap: DD{20,21,32}'s alternative signed section has
   positive projected turns although the prescribed native result is triangular; validity
   alone does not select policy. Coordinator rejected frozen setback-tuple whitelisting.
   Trace the actual native geometric decision for this case and DD{19,22,33}, test symmetry
   and perturbations, and reproduce a demonstrated deterministic construction policy only
   under its hypotheses. A heuristic is not a uniqueness proof. Unresolved cases stay
   rejected while established plans/shared assembly proceed; no invented selector.
   New isolated DD{19,22,33} evidence proves actual pivot ambiguity: six exact-QA outputs
   split between pivot19 bilinear and pivot22 triangle, with 48 strict material disagreements
   surviving +/-2% perturbations. Coordinator authorized separate bounded A5 preference
   admission under the user's existing explicit-preferred-edge policy, using the existing
   ChamferCornerPreference API. A6 guards unchanged; absent/invalid/unused preferences
   reject immutably without fallback. Support only material-validated chosen rules. Explicit
   input-bound replay annotation may select the intended DD shape; unannotated DD rejection
   remains and annotated gains are separate. Minimal modeling/replay routing and tests are
   now within ownership if needed; no new policy enum or broad API change. DA remains excluded.

I2 research (target/i2-contract-20261104/REPORT.txt) derives a torus, base/cap cylinder
and nonlinear outer-cone/cap canal; F9/G8/G4 two-cylinder construction does not apply.
The ideal inner seam reaches the radial cap before the cone contact and needs a residual
torus meridian. Source volume and isolated inner removal are independent; total retained
material is now specified by the outer contract at target/i2-outer-contract-20261104/REPORT.txt.
It derives unsquared monotone outer-seam/base-residual/remote roots, the earliest interior
remote contact, eight oriented face loops and overlap-aware material integration. The 192
ideal witness checks pass; numerical quadrature is convergent but not a rigorous error bound.
Native exact QA and the original 108 local patch witnesses remain separate evidence.
Captured/procedural retained-library rejections preserve input, but are not fresh-parent
proof. Independent ideal-contract review and actual-support perturbation, root stability,
fitting and topology-separation budgets remain required. Ideal review at
target/i2-ideal-review-20261104/REPORT.txt found no blocking geometric defect. Carry
F_t/Z_t/D0 >= 1 at endpoints (strict only away from zero sine) into implementation.
Use its regularized volume evaluator, with an explicit numerical oracle allowance;
original quadrature digits are not exact. Analytical flux closure is established,
not an independently executed parametric face-flux quadrature. No I2 production
admission, geometry gain or performance claim. Budget research closed scratch-only at
the user stop request: target/i2-representation-budget-20261104/REPORT.txt records
actual cone displacement bounds and a candidate tensor canal fit. Shared trimming
curves still lack complete derivative, endpoint, stored-control and same-parameter
bounds; full admission remains rejected. Resume only on a new user request.

### B. Fillet junctions on curved supports

Owns `truck-shapeops/src/fillet/planar_fillet.rs`, `planar.rs`.

1. **Broader cubic/quartic cases.** Bounded G2/G6 are committed; H1 is implemented and
   reviewed in the pending batch. Extend only with independently derived geometry and parent-failing
   regressions. Preserve `tests/extrusion_remote_junction.rs`, `quartic_remote_junction.rs`
   and their independent oracles. G2/G6 native comparisons remain failed.
2. **G8, then G4**, after bounded F9 integration: two selected straight edges meeting at an extrusion corner with
   the curved cap edge unselected. Not the G2 topology and not a three-edge sphere. Exact
   source geometry and an oracle plan are in
   [notes/fillet-junctions-f9-g4-g8-i2.md](audits/fillet-chamfer/notes/fillet-junctions-f9-g4-g8-i2.md).
   F9 has bounded geometry approval and completed mesh/resource validation. Valid matched
   native timing stays blocked by exact native QA; native comparison/classifier issues remain.
   G8/G4 require their own contact-order/material contracts; do not widen F9 admission.
   G8 is implemented and reviewed with direct parent proof, original supports, shared
   crease/residual arc and independent cubic clearance checks. Its sampled comparison,
   material and exact native QA gates pass; matched timings are recorded above.
   G4 contract delivered: target/g4-contract-20261104/contract.txt. Two-cylinder crease
   applies with oblique-axis corrections; seam crosses the radial turning point and cap
   terminates first, with residual cap arc entirely beyond the turning point. Independent
   volume/flux and 806 native material witnesses agree; exact native QA passes. Retained
   library probes reject six radius/order controls immutably but are not fresh frozen-parent
   proof. Implementation must establish that proof, per-request interval disk containment
   and bounded remote-termination fitting. Separate private g4.rs admission; F9/G8 unchanged.
   G4 candidate is now implemented, but captured STEP cap-profile admission rejects:
   representation deviation about 5.5172e-11 exceeds arithmetic allowance 4.6359e-11.
   Nine integration tests and one certificate test, scoped Clippy/format pass; fresh frozen
   parent positive fixtures fail. F9/G8 bytes unchanged. No captured-output gain: full suite,
   docs, replay tests/full486 and constructed-output QA remain pending. Follow-up must derive
   and propagate an imported-representation error budget rather than widen a factor or
   substitute geometric eps blindly. Evidence: target/g4-implementation-20261104/REPORT.txt.
   Exact decimal investigation (target/g4-representation-20261104/REPORT.txt) proves
   a real represented-support discrepancy, including direction error; height adjustment
   alone cannot restore the exact family while retaining supports. Ideal polynomial
   quotient positivity does not certify perturbed tangency. Follow-up must compute on
   actual represented coefficients or explicitly propagate support/domain/contact errors,
   return contact-root bounds to construction, and sum all fitting/arithmetic terms within
   the existing budget. No guard was widened. Fresh capture still rejects immutably and
   yields no STEP; full validation remains pending. Actual projected quartic recovery,
   interval contact roots consumed by construction, represented-profile containment and
   actual remote derivative/stored-span bounds are now implemented in g4.rs. Two unit and
   nine integration tests, scoped Clippy/format pass. Remaining work is sweep/domain/plane
   discrepancy and full inverse/evaluation/control-construction budget propagation; fresh
   capture still rejects immutably. Evidence: target/g4-contact-20261104/results.json.
   G4 now constructs the captured .001 case: exact native/output QA, 806 material checks
   each and sampled comparison pass. Final budget sum 4.596e-7 is below allocated eps/2
   1.101e-6 with unchanged eps; sweep/frame/planes/cap representation, root, inverse,
   endpoint/stored controls, evaluation and truncation charged explicitly. Original supports
   retained; bounded approximate-family claim, not exact perturbed tangency or formal all-FP proof.
   Final 10 integration/2 unit tests, Clippy/smoke pass. Pre-final broad603/7docs/77replay
   are evidence only; combined final freeze still pending alongside A5. Pinned full486
   remains365/343 with no lostchecks, G8 and annotatedA6controls pass. Frozen imported
   regression fails; F9/G8 unchanged. Independent review now running; performance pending.
   Evidence: target/g4-budget-20261104/final-round/REPORT.txt and identity.json.
   Independent review approved exact final source e26f548e...: represented-contact quotient,
   domain/plane/frame/root/Hermite/evaluation accounting and topology/history checked;
   rational truncation oracle rerun. All nine manifest identities match, including F9/G8
   and pinned replay. Raw replay check comparison independently confirms zero losses.
   No concrete omitted budget term or loose gate found. Approval remains bounded approximate
   family, excluding A5 delta, final combined-checkout acceptance and performance. Guard
   rejection tests prove immutable public failure, not exclusive internal guard causality.
3. **I2** (quarter-revolved triangle, cone network; same note), then triage the rest:
   `blend/simple/{H3,H5,H9,I4,I8}`, `blend/simple/X1/002`, `blend/tolblend_simple/{D3,D5}`,
   `bugs/modalg_4/bug828`, `bugs/modalg_5/bug25657_1`, `bugs/modalg_7/bug30940_{2,4,5,6}`,
   `blend/complex/G9/002`.
4. F5 follow-up, only once a failing case exists: the extrusion branch samples blend
   orientation at chart point (0.5, 0.5), outside the fitted domain on short bodies.

### C. Unequal radii at a junction (6 cases)

Owns `truck-shapeops/src/fillet/planar_edge.rs`; `fillet/modeling.rs` is *shared* with B
(dispatch). Bounded `blend/simple/P9` is implemented and reviewed in the pending batch,
including numerical-error and rational-weight guards. The historical investigation is in
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
3 were the A1 corners, now passing in the pending batch; H7 and F5 were added later).

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

- A6/A8/A9 tied planar chamfer corners: require an explicit preferred edge, adopt incident
  selection priority, or retain rejection: DECIDED by user — explicit preferred edge,
  preserving selection-order independence. API design is recorded above; implementation remains open. Evidence is in
  `target/a6-quad-contract-20261103/contract.txt`.
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
| G05 | Curved and closed spline spines | partial | G1 discrepancy, F9/H1 (stream B), the six `bug30940` cases |
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
| 2026-10-26 | `b0ba3f2b` | STEP knot/export fixes, G2 and C2 junctions; chamfer replay 357 / 332; G2 25.46 ms vs OCCT 29.70 ms with rolling-ball geometry preserved |
| 2026-10-22 | `9f97532a` | Torus corners; full run 480 construct / 437 pass |
| 2026-10-22 | `9e3ce08a` | Mixed curved corners; F1 passes, H7 constructs |
| 2026-10-24 | `5a4e3bc6` | Exact knots and STEP masters; full run 482 / 438 |
| 2026-10-24 | `834ee9f5` | Nine skipped native cases captured as a separate cohort |
| 2026-10-24 | `d3858852` | Elliptical extrusion corners; F5 constructs (discrepancy 0.107, OCCT reference suspect) |
| 2026-10-25 | `2cf44b16` | OCCT three-edge chamfer corners; chamfer replay 351 construct / 326 pass, unchanged |

Older trackers, handoffs and status reports were removed on 2026-10-25; they remain in git
history (`d86c6cfb` and earlier).

Current ellipse-writer closure remains incomplete/admission-disabled. Its stored-control
and phase certificates compile, but inward serialized endpoint knots, reader-projected
trim coverage, normalized frame differences and cache/fallback evaluator bounds remain
open. Seven helper tests, 142 explicitly scoped preservation tests, six doctests and
Clippy pass; the parent and candidate production graph positives remain red. Evidence:
target/g4-g8-ellipse-writer-closure-20261105/REPORT.txt. Split remaining reader-domain
and evaluator proof work in scratch; no production QA or performance acceptance.

I2 complete-boundary scratch patch is ready for application/validation, not approval:
target/i2-whole-boundary-20261105/delta.patch (base42a6c517, candidate4fed7b1d).
It adds whole-source quarter-profile recognition, all36 paired support rows and
expanded rational/Hermite fit transfer. 62 lightweight checks only so far. A new
validation owner receives the exclusive heavy slot to apply/debug, prove incoming
failures, run complete private material/STEP controls and bind final ledgers/runtime.
Public admission and deployment eligibility remain closed; STEP reader work stays
scratch-only during this source freeze.

Ellipse reader-domain scratch patch now encloses actual gp_Ax2 frames, certified phase
shifts and outward serialized knot coverage, but Project/extrema and G8 zero-seam
branches remain unclosed (target/ellipse-reader-domains-20261105/REPORT.txt).
Coordinator source inspection found a narrower candidate route: emit a parameter-master
TRIMMED_CURVE around the unchanged analytic ellipse; ShapeAnalysis_Curve::Project
checks bounded-curve endpoint distances before ProjectAct, even with AdjustToEnds=false.
Validate exact trim/map/vertex bounds and normalization before adopting this route;
no imported acceptance is claimed. I2 retains the heavy slot; writer work remains scratch.

I2 whole-boundary candidate now compiles and passes 26 library/28 private integration
material/STEP tests, 180 source/input/candidate-bound support rows, Clippy and format.
Final source d0628360, executable764cc291. Six expected incoming regression failures
and one preserved pass are recorded, with test-adapter/projection-witness limits.
Evidence: target/i2-whole-boundary-validation-20261105/REPORT.txt. Slot released;
independent follow-up gate review is required. Public positive remains NonPlanarFace;
deployment eligibility and unrelated STEP graph failure remain open.

OCCT cubic pcurve evaluator helper now has independent bounded approval:
target/ellipse-reader-evaluation-review-20261105/REPORT.txt (998 exact coefficient
checks, source-path and retained disassembly analysis). Consume only under its knot,
selection, arithmetic and mutation-exclusion assumptions. Final reader mapping,
endpoint hulls/frames and complete export inequality remain open; no geometry
admission follows from helper approval. Bounded-analytic-trim integration continues
in scratch; I2 source is frozen for independent review and heavy slot is free.

Bounded-analytic-trim writer endpoint patch and mesh-free diagnostics are ready.
Coordinator grants scoped application of out/{geometry,topology,mod}.rs and exclusive
max-eight-job build slot for endpoint/graph validation. I2 d0628360 remains frozen
for read-only review; out/modeling.rs dd7b9e02 remains unchanged. Final writer admission
is disabled pending complete budgets/runtime/mutation exclusions.

Independent I2 whole-boundary review approves closure of all four private-gate findings
under the reviewed scalar contract: target/i2-whole-boundary-review-20261105/REPORT.txt.
Public routing still needs complete-solid single-shell/selection/radius validation and
a supported build/execution policy with per-calling-thread FP eligibility. Do not equate
the private runtime assumption token or startup MXCSR capture with deployment approval.
Prepare minimal routing/policy in scratch during the STEP build; no public enablement yet.

Ellipse bounded-trim endpoint changes are applied in out/{geometry,topology,mod}.rs.
Six original fixtures pass; the Python evaluator bridge gives a conditional total
within 5e-8. Final library has 26 passes and the intentionally red production graph;
Clippy, formatting and six doctests pass. Evidence: target/ellipse-bounded-trim-20261105.
Next: independent endpoint/graph review and typed evaluator integration, preserving
reader mutation and runtime eligibility as explicit gates. Only the typed evaluator
owner receives the heavy build slot; I2 and out/modeling.rs remain frozen.

I2 public routing/runtime proposal is scratch-only: target/i2-public-route-20261105.
Review the feature-off default, whole-solid/selection/history routing, per-thread
fegetenv boundary and native Fedora deployment proposal before application. No
packaging activation, public admission or global-green claim is authorized by the
private gate approval. Independent design review runs without builds or live edits.

Independent endpoint/graph review approves archived geometry29c53e5b under its
stated assumptions (target/ellipse-bounded-trim-review-20261105). It verifies all
71 artifact hashes and six original endpoint fixtures. Added the required
DisplayByStep migration note to CHANGELOG; concrete PiezaCAD points need no change.
Typed total must include source_trim_error jointly, not as a separate threshold.
Reader mutation/array association and final runtime mapping remain open; assign
a parallel scratch-only source-path audit while typed implementation owns builds.

I2 routing review found consumer duplicate/side validation could be bypassed and
its allow_i2 flag did not constrain the later ordinary kernel call. Coordinator
corrected the scratch candidate at target/i2-public-route-corrected-20261105: private
family helper, ordinary whole-solid API, raw validation before chain expansion,
and complete inter-chain junction dispatch before grouping with no subset retry.
Both patches apply cleanly; compilation and actual route regressions remain pending.
No live source/deployment activation. Native Fedora is the provisional supported
build approach; broader packaging is unnecessary for disabled-source validation.
STEP typed evaluator retains the exclusive heavy build slot.

Typed ellipse evaluator/combined ledger is implemented at geometry83d257bf,
topology6baadc3c (target/ellipse-typed-evaluator-20261105). Six actual fixtures
majorize the rational helper within unchanged5e-8; source_trim_error participates
in the same total. 150 scoped tests/six doctests/Clippy/format pass; production
graph remains explicitly red. Independent typed review and reader mutation audit
run read-only. Writer sources freeze while I2 receives the exclusive max-eight-job
build slot for corrected feature-off public route application and enabled-test
validation. No launcher/deployment activation, timing, or public acceptance claim.

Typed ellipse review gives bounded approval (target/ellipse-typed-evaluator-review-20261105).
Reader audit found all six complete retained arrays/associations preserved, but
.U. closedness, neighboring-wire healing and two additional opposite-side pcurves
need explicit handling (target/ellipse-reader-mutation-20261105). Coordinator
prepared the narrow .F. formatter correction/regression in scratch
target/ellipse-open-pcurve-20261105; no live writer edit during I2 freeze.
Use an exact post-transfer representation/association guard for the bounded
validation consumer, rejecting changed arrays/ranges/frames or missing sides.
This is not an unrestricted guarantee for arbitrary STEP readers/healing settings.
All emitted associations need actual typed ledgers. Admission stays closed pending
this integration and runtime binding; I2 retains the exclusive heavy slot.

Post-transfer guard scratch is prepared at target/ellipse-post-transfer-20261105.
It resolves source transfer binders, requires final-shape membership and complete
edge/face/pcurve association, and checks arrays/ranges/frames before and after a
callback under exclusive ownership. Native guard and nine mutation controls are
uncompiled. Eight original-input topology bindings are verified; actual eight-side
Rust ledgers and the STEP/normalized-frame binding stream still need execution.
Coordinator .F. formatter patch and eight-side fixture patch jointly apply cleanly.
Queue their application, native compile/controls and final binding for the writer
slot after I2 releases. No numerical, whole-STEP or runtime acceptance inferred.

Corrected I2 route is applied with defaults off (target/i2-public-validation-20261105).
Enabled34 integration tests, final app249 library tests, fresh180 paired rows and
full486369/347 pass. Feature-on/off checks and owned Clippy pass. Final source
32fa66f1 and executable snapshots are recorded; old mutable Cargo executable was
not archived before rebuilding, so historical executable-byte preservation is
not claimed. Independent route/FP review remains required; no deployment activation.
Broader run exposed eight STEP preparation failures plus B1 replay STEP failure;
identical failures occur with incoming I2 sources/current writer. These remain
open, not waived as pre-existing kernel behavior. Attribute current writer changes
against their own incoming source before combined acceptance.

Writer receives exclusive max-eight-job slot for explicit-open correction, eight
actual association ledgers, native guard compilation and mutation controls, exact
source/STEP/frame binding and guarded QA. I2 source/test/app files freeze for review.
Production writer stays disabled until final integration review/runtime eligibility;
G4 native tolerance-loss roundtrip remains independently unresolved.

Independent final I2 review approves the default-disabled source within scope
(target/i2-public-final-review-20261105):3821 artifact hashes,11 source pins, both
new executable snapshots and180 paired rows verified. Activation remains a finite
supported-artifact task, not a private-geometry proof restart. Prepare the minimal
native Fedora build/runtime-selection checks and reachable-hook audit in scratch;
no launcher/default/deployment edits while STEP owns the build slot. Coordinator
will assess concrete final artifacts under existing implementation authorization.

STEP attribution corrects the earlier I2 comparison: incoming B1 failed reading
--exact as a preference filename, not preparing STEP. Preserved logs/report have
a separate B1-COMPARISON-ERRATUM.txt; no identical incoming B1 failure is claimed.
All eight tests and B1 passed earlier writerc4a37805. Current support-fit regression
window remains open. Unsupported supports already bypass the strict certificate;
no broad bypass fix is justified. Prepare mesh-free capture/instrumentation and
strict bounded-chart/guide-degree corrections in scratch; actual subreasons need
execution after ellipse guard releases the build slot. Preserve5e-8 guide/join/
final coefficient checks and original G4/G8 support improvements.

Supported I2 artifact candidate is scratch-only at
target/i2-supported-artifact-20261105: build recipe, eligibility helper, explicit
run-supported entry point, lightweight negative tests. Ordinary run.sh remains
feature-off; no approved manifest or installed artifact exists. Review helper
input/runtime identity, build-option enforcement and fail-closed launch behavior
before any final-artifact build. Avoid broad security/packaging machinery. Final
GUI code/runtime and reachable-hook/ownership evidence remain finite open checks.

Strict support-fitting candidate/capture package is prepared at
target/step-support-regression-fix-20261105. Root inspected the narrow chart clamp
and degree-elevation budget; no epsilon/domain extension or return-check bypass.
Patch remains unapplied/uncompiled and named failures unresolved. Next heavy slot
after ellipse guard joins processes goes to same-dependency c4a/b293/dd7/candidate
mesh-free captures and mechanism tests, then evidence-driven application/full tests.
Ellipse owner notified; no interruption or overlapping build authorization.

Eight-side staged ellipse validation completed (target/ellipse-guard-validation-20261105).
All8actual ledgers fit5e-8 after axis-L1-aware interpolation allocation; guarded
G4/G8 exactQA/all24pairs/material806/770/repeatability pass. All9mutation controls
reject in both solids;153scopedtests+6docs/Clippy/format pass. Geometry168b07f2,
topologyc246bb14 freeze for independent integration review. Production stillNone;
nativeG4roundtrip failed. Heavy slot released to strict STEP preparation diagnosis
and correction in out/modeling.rs; no overlapping builds or writer-graph changes.

Supported artifact helper review approved preparation with one inventory hole.
Coordinator fixed exact before/after source and toolchain inventories in scratch
target/i2-supported-inventory-fix-20261105, also preserving byte checks and guarding
provenance record changes/escaping links/overlapping mutable paths. Seven lightweight
tests pass; identical added-input regression fails incoming in four subcases.
All build subprocesses were mocked; no artifact build/manifest/activation occurred.
Use corrected helper/recipe, retaining original final build/runtime obligations.

Independent final ellipse integration review approves the bounded serialized
agreement and exclusive guarded consumer (target/ellipse-guard-final-review-20261105).
All8ledgers/144framecoordinates/completeassociations and runtime evidence verified.
Production remainsNone; arbitrary readers and deployed runtime are not approved.
Prepare the smallest supported writer eligibility/activation delta in scratch,
including construction-to-format lifetime and default-off behavior. Reuse existing
FFI scalar policy without adding an OCCT product dependency or a generic framework;
the exact post-transfer guard remains validation-consumer evidence. STEP fitting
owner keeps the exclusive build slot and live source ownership.

Ellipse production-route candidate is prepared in
target/ellipse-production-route-20261105, not applied or compiled. It adds
default-off supported-lane admission, exact built-in type gating, stable-index
construction/format eligibility, shell buffering, and fallible app exports.
Coordinator inspected the patch; independent review of formatting lifetime,
type/API scope and failure propagation precedes application. STEP fitting keeps
the exclusive build slot. Final artifact/runtime selection and fresh guarded
G4/G8 exports remain required; native G4 roundtrip stays failed.

Production-route review found one default-off staged-test regression. Coordinator
corrected it in target/ellipse-route-review-fix-20261105: the identity-only test
uses private body formatting with all assertions retained; optional whole-solid
staging now explicitly requires the eligible feature build. New scratch tests
cover ineligible construction, delayed controls through two shells/void solid/
assembly, sink errors, and both app exports retaining destination bytes after a
real formatter error. App serialization shares one private fallible writer.
Nightly parsing/formatting and both patch applicability checks pass; incoming
live pins are unchanged. No Rust compilation or runtime pass claimed. Add the
remaining deceptive curve/surface metadata controls and execute incoming/mutant
proofs during the next exclusive slot, after STEP fitting releases it. Original
candidate/review artifacts remain untouched; no production activation.

Strict STEP fitting correction completed at modeling a0773491
(target/step-support-regression-validation-20261105). All eight named failures
and correctly invoked B1 pass; 700 enabled shapeops tests, 157 STEP tests plus
the known disabled-graph failure, 77 replay tests and 13 doctests pass. Full486
remains369/347 with no lost checks. Finite polynomial/conic chart witnesses are
clamped with full displacement charged; lower-degree elevation is charged.
Independent correction review proceeds read-only. Heavy slot is released and
assigned to application/compile validation of ellipse-route-review-fix-20261105.
That owner may edit only the reviewed writer/runtime/Cargo and app export files;
modeling a0773491 and I2 remain frozen. Refresh combined guarded G4/G8 evidence;
defaults/deployment stay off. Coordinator owns the Breaking migration entry.

Independent strict-fitting review approves modeling a0773491 and its fixture
without required changes (target/step-support-correction-review-20261105).
Full displacement/elevation charges, unchanged strict guards, 595 capture pairs,
70 actual parent-failing curves and369/347 replay parity were independently
checked. The eight fitting regressions and correctly invoked B1 are resolved
within this scope; preserve the historical B1 invocation erratum. Ongoing ellipse
route validation is separate, as are deployment and performance acceptance.

User closeout instruction: finish current working points, then stop. The active
ellipse application/validation owner is completing only its existing scope and
joining all processes. No new research streams, measurements, deployment,
supported-artifact installation or commits are authorized by this closeout.
Coordinator will inspect the final evidence and record remaining limitations;
default-off features and historical failures stay explicit.

Closeout: ellipse application/validation finished in
target/ellipse-production-validation-20261105. Coordinator verified all eleven
final source hashes, default-off feature declarations and both repository diff
checks. Enabled STEP167/167 passes; default161 passes plus the intentional graph
failure. Full486 stays369/347, all30C2 sequences/three preference controls and
fresh guarded G4/G8 QA/material/repeatability/18 mutation rejections pass.
Activation remains blocked: ellipse-enabled shapeops has17 downstream Truck
reader failures and replay has2 (TRIMMED_CURVE support); ellipse-off controls
pass. G4's actual guarded consumer reaches scalar argument12.184038203338222,
outside the reviewed[-8,8] runtime contract. That failure is retained, not waived.
Final supported-artifact/runtime audit and original native G4 tolerance-loss
roundtrip remain unresolved. Defaults and deployment are unchanged, no commits.
All assigned processes have joined and the exclusive slot is released. Work is
stopped at the user's request; do not launch follow-up work without a new request.

2026-11-06: user requested continuing until told to stop; the previous stop is
lifted. Resume with Truck TRIMMED_CURVE import support for the guarded writer
and attribution of G4's out-of-contract scalar call. Defaults remain off and
no commits/deployment are authorized. Reader implementation owns the exclusive
heavy build slot; scalar attribution uses retained artifacts and scratch only.
Preserve the complete prior validation and frozen writer/I2 sources; resolve
the actual activation blockers without loosening numerical or reader checks.

G4 scalar attribution completed in target/g4-scalar-domain-20261106: OCCT
line/ellipse wire-healing alignment adds one period after a one-ulp endpoint
mismatch. Certified writer/control hulls stay within[-8,8]. A separate rational
[-16,16] scalar proof proposes the same2^-48 component error, not a geometric
tolerance increase. Independently review that proof and the smallest consumer
validation policy before changing any interposer. Prefer a uniformly proved
consumer scalar contract over callsite/phase machinery if it preserves all
geometric domains and checks. Existing strict abort/evidence remains failed.

Independent g4-scalar-review-20261106 approves uniform[-16,16]2^-48 scalar
accuracy under existing premises and recommends the simple separate consumer
policy, without callsite exceptions. Coordinator prepared the one-predicate
interposer delta and boundary/control probe in target/ellipse-consumer-runtime-20261106.
No builds ran: reader implementation retains the exclusive slot. Native policy
negative tests and full guarded G4/G8 validation follow its release. Writer8,
geometric domains8 and I24 remain unchanged; no deployment approval.

Trimmed conic reader implemented (target/trimmed-conic-reader-20261106): bounded
singleton parameter trims preserve explicit branch/orientation; all17 former
shapeops reader failures and replay77 pass, with full486369/347 unchanged.
Independent review is pending. Heavy slot released and assigned to the separate
consumer-runtime16 validation, with frozen production sources. No defaults or
deployment change; retain original strict8 abort and native G4 roundtrip failure.

Separate consumer16 validation completed in target/ellipse-consumer-validation-20261106:
84 boundary/environment probes, guarded G4/G8 exact QA/all24pairs/material806/770,
repeatability and18mutation rejections pass. Fresh ordinary writer exports match
DATA bytes;8ledger hulls/144frames verified. Strict8 originalG4abort retained.
Independent evidence review pending; heavy slot released. Serial interposer is
an audit tool, not a production/thread-safety policy. Final application-artifact
build/selection/ownership audit remains separate, after reader review settles.

Reader review found shifted-full-turn, large-phase and overflowing-placement
admission defects. Coordinator corrected both owned files; all15 focused tests
pass, including the three incoming failures and positive reversed interior trim.
Evidence in target/trimmed-conic-admission-fix-20261106; broad validation and
independent numerical review follow. Root's saved parent-test-binary was the
other feature variant, not the14-test failing binary; preserve it but do not
claim correspondence. Reproduce final fixtures in an isolated incoming source
tree to obtain correct final parent binary evidence. Heavy slot released by root.

Consumer16 review accepts retained results but requires a successor harness:
claim empty output before writes, explicit effective environment, integrated
expected-pin checks before launching. Historical originals remain untouched;
root will make these small harness corrections, not delegate new research.

Reader numerical correction received bounded source approval; root fixed the
remaining test-only gap by adding a separate valid2Dplacement large-phase test
and small-phase positive controls. Final testSHA1114ac7c; production20181ade
unchanged. Active validation owner has final fixtures for isolated parent proof.

Root prepared successor consumer replay in target/consumer-harness-successor-20261106:
exclusive new output claim, explicit effective environment, pinned executable/
preload checks on actual launch path, before/after source/dependency verification
and exclusive final manifest. Six mock integrity tests pass; native replay is
not executed while reader validation owns the heavy slot. This replays immutable
reviewed binaries, not a compiler rebuild; retained original evidence untouched.

Final reader validation completed:16focused tests pass; isolated incoming has
four intended failures including valid2Dlargephase. Identical production bytes
pass700shapeops/replay77/B1/13docs/Clippy; full486369/347 unchanged. Source review
approved correction, final test-gap proof now supplied. See trimmed-conic-admission-validation-20261106.
Root executed consumer-controlled-replay-20261106-01 with successor harness:
84probes, guarded QA/material/repeatability and18mutation rejections pass.
Actual launch wrong-pin controls reject; all effective environments recorded.
Manifest verified; a second invocation rejects before modifying any evidence.
Heavy slot released. Next is frozen application audit-candidate build and finite
source ownership/hook audit; no installation, defaults or deployment activation.

Source ownership audit completed (target/supported-app-ownership-20261106): no
concrete aliased-input write; actual setter provenance and synchronous I2/STEP
publication traced. Bind its source pins to final frozen build. Remaining checks
are actual binary/runtime signal chaining, returning context changes, allocator/
scalar bindings and renderer-loaded modules. Final builder received these finite
requirements; no generic locking/deep copy or deployment approval follows.

Both combined-feature audit artifacts built in target/supported-app-build-20261106:
exact inventories and ownership pins match; ten helper tests and argument-only
smokes pass. No numerical operation, renderer policy or deployment approval follows.
Continue with final-binary scalar/folded-constant and production hull mapping,
and separate observed numerical-path providers/signals in isolated runtime.
Preserve frozen binaries and default-off features; do not manufacture approval
records. Native G4 roundtrip and historical failures remain unchanged.

Final static map in target/supported-app-numerical-map-20261106 binds evaluator
call edges and seven rational constant checks to both frozen app artifacts.
No production ledger execution is claimed; gates1/2 remain open. Exact runtime
capture requests were supplied to the active runtime owner. Continue the finite
derivative-helper and folded-constant boundary rather than repeat the same map.

Final cadc runtime audit passes ordinary rebuild/export but I2 rejects. Root
localized the exact36-row production ledger: DC/base-plane row0 has projection
allowance1.2985843e-5; other35 rows fit1e-9. See target/i2-final-document-rejection-20261106.
Investigate correlated implicit-root interval overestimation for the ideal DC
z=0 trace, retaining stored spatial/error charges and actual plane displacement.
No budget relaxation; require parent-failing plane-chart/app controls. Original
final binaries/evidence remain immutable; defaults stay off.

Helper continuation (target/supported-app-helper-closure-20261106) resolves finite
quotient/composition mechanisms but does not close gate1. Its source finding is
that the writer scans elementary descriptors before edge eligibility, so generic
Fillet/Sphere extraction cannot be claimed globally unreachable. I2 accepted
D0 types are narrower; its certificate does not consume arbitrary high-order
surface derivatives. Keep these scopes distinct; do not require unrelated
high-order proofs or claim a new numerical defect without a reachable counterexample.
Current active implementation is only the root-localized DC/base-plane budget
correction, with heavy slot assigned and default-off activation preserved.

Ideal DC zero-coordinate correction completed (target/i2-plane-trace-fix-20261106):
only actual ledger row0 changes, other35 bit-identical; incoming fails final
mechanism/public/app regressions. Root reviewed sign-bracket/monotonicity proof
and retained plane displacement/spatial charges.37 Truck and14 app controls pass.
Root explicit STEP rerun also37 passes (target/i2-plane-trace-step-20261106).
Erratum: original step-test WAS active via default feature, not a coverage gap.
New i2 source370ee6c3 requires refreshed final app artifact identities; prior
frozen binaries remain immutable historical evidence. No activation/default change.

Refreshed final cad-app195fbbab/cadc0630499e built and frozen in
 target/supported-app-refresh-20261106. Exact saved I2 document now succeeds
through production check/export; all36 executed budgets fit, other35 unchanged.
Discriminating child-only rounding negative rejects and clean positive recovers.
Fresh exported I2 STEP exact QA fails imported edge15/face8 same-parameter check:
3.1306699e-7 exceeds stored3.1291271e-7; nearest-surface distance2.48e-14.
Attribute raw/prepared/imported representations before any correction; retain
ordinary-control edge29..32 failures and nativeG4 roundtrip. No activation.

I2 STEP attribution completed (target/i2-final-step-attribution-20261106): imported
edge15/face8 maps to CS, raw affine UV on stored canal boundary. Preparation
replaces it with a single Hermite cubic; reader UV error is only about1e-14.
Scratch exact stored-isocurve replacement passes QA without tolerance change.
Implement bounded exact boundary restriction in preparation with original range/
orientation and numerical checks; avoid a generic pcurve export framework.
Four thin-boundary native material mismatches persist in original and control;
keep them failed pending attribution. Span-aware volume corroborates CS error;
CLI coarse-mesh volume is separate and not an exact-material measurement.

Bounded PCurve boundary restriction implemented in stepio/out/modeling.rs4a3ffeff
(target/i2-exact-isocurve-20261106): affine UV on clamped polynomial boundary,
original parameter trim retained. Parent regression fails; enabled186STEP+6docs,
37I2tests andClippy pass. Fresh scoped actual document export passes exact QA;
original remainsfailed, other STEPentities unchanged moduloCSreference shift.
Independent rounding/admission review required before finalartifact refresh.
Four material discrepancies and ordinary-control QA remainfailed and separate.

Independent boundary review found unsafe mapped-span arithmetic outside the retained
I2 fixture (target/i2-isocurve-review-20261106). Root correction now rejects collapsed
and unsafe mapped/cut spans, uses fallible spline construction, and checks final
controls. A valid-source regression panics on incoming production and passes the
correction; focused controls pass. Broader allowance proof remains open: do not
interpret finite arithmetic or the retained fixture certificate as universal
PCurve evaluation certification. Validation evidence is being collected in
target/i2-isocurve-arithmetic-20261106; root holds the heavy slot.

Thin-witness attribution (target/i2-thin-material-20261106) supports ideal and
stored-coefficient IN at both unchanged points. P1 OUT is rejected classifier
fallback after repeated ON-trim ray retries; P2 ON follows requested1e-8 base
tolerance semantics. All four original rows remain FAILED. The lower-level DC
knot ON classification is not repaired; no Truck material-removal defect is
demonstrated by these probes. Native G4 roundtrip and ordinary-control QA stay
separate. All attribution processes joined.

Root transformed-arithmetic correction validation complete:187 enabled STEP
tests+6docs,37 I2 integration/material/STEP, Clippy/format/diff pass. Final
valid-source regression fails incoming4a3ffeff inside mapped spline construction;
original1e-308 exploratory source was rejected earlier and is not counted as
proof. Four original material discrepancies remainfailed with comparator
attribution. Universal allowance proof and final supported-artifact refresh
remain open. Evidence:target/i2-isocurve-arithmetic-20261106. All processes
joined; heavy slot released.

Root replaces the unproved boundary epsilon multiplier with an explicit
stored-coefficient certificate (target/i2-boundary-bound-20261106): outward
normalization/map enclosures, paired knot-cell transport bound, and interval
Boehm restriction checked against actual output controls. Scope is real stored
curves, not full floating PCurve or arbitrary reader evaluation. Eight focused
tests and exact-rational checks of4242 transport samples/1272 actual control
enclosures pass; existing serialized424-pole result is reproduced after its
existing FloatDisplay rounding. Raw-vs-serialized exact identity was an invalid
comparison and its failed exploratory check remains recorded. Broad scoped
validation underway; no activation/performance claim. Root holds heavy slot.

Explicit boundary coefficient check validated:191 enabled STEP+6docs,37 I2,
Clippy/format pass. Final modeling c1835b08 and private boundary7742d03a;
source/binary identities and actual424-pole interval audit are retained under
target/i2-boundary-bound-20261106. Explicit proof replaces heuristic allowance
for real in-memory curves only; no floating-evaluator/reader/runtime approval.
Independent proof/code review and supported-artifact refresh remain gates.
All spawned processes joined; root heavy slot released.

Catalog publication cleanup: generated catalog JSON/HTML are ignored and the
generator emits execution summaries with links to full evidence. All historical
case/capture outcomes remain in the index; manifests and archives are untouched.
Original pre-cleanup commits are retained locally at
backup/pre-catalog-cleanup-20261106, with original catalog bytes and validation
in target/catalog-cleanup-20261106. Only unpublished master history is being
rewritten to remove those two generated paths; published ancestry is preserved.
