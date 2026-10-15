# Physical blend capture and replay

This tool records actual input solids, requested blend parameters and physical edge curves
from OCCT's `blend` and `chamf` commands, then tries equivalent Truck constructions. It does
not use ordinal edge IDs across kernels. Instrumented runs are separate from the
unchanged native corpus executions under `../runs`; neither successful construction
nor sampled shape agreement establishes complete feature parity.

The DRAW plugin copies shapes before STEP export. Its Tcl wrapper preserves the
native command's caller scope, return value and error options even when capture
fails. Captures contain BREP/STEP input and result shapes, selection polylines at
1e-5 deflection, native validity, solid counts, integrated volume and area. Logs,
source/binary hashes and physical selections accompany each replay. New runs also
record the actual kernel HEAD, tracked worktree changes and hashes of Rust sources
and Cargo manifests/lockfile; the catalog snapshot is not assumed to be the runtime
kernel revision. Replay reports retain the stage of a panic or failure.

The replay uses the app's existing closed-edge/face splitting operation before
mapping STEP geometry into modeling types. It checks input consistency, matches
physical edges in both directions within 1e-4, rejects detected overlapping
matches, and permits split representations. Equal-radius selections first use
the solid-edge route, then the wire route when the selection is one unbranched
chain. Candidate polyline chords must be covered by the captured selection tolerance
tube; endpoint-only matches cannot select a straight chord across a curved edge.
Input import, selection mismatch, unhandled intent, operation failure,
panic, timeout and constructed output are recorded separately.

Successful output is checked for geometric consistency, closed tessellation,
unchanged input and STEP preparation. The native comparator reads the Truck STEP,
checks validity, counts solids, integrates volume and measures distances in both
directions between sampled boundaries. It samples 25 points per edge and an
11×11 UV grid per trimmed face. Distance targets are face compounds, so a point
inside a solid is not automatically assigned zero boundary distance.

## Reproduce

Requires the same OCCT 7.9.3 runtime/SDK, Tcl 9 headers, DRAW, sway and Xwayland as
the recorded native runs. The build script uses this workspace's Fedora SDK layout.

```sh
bash audits/fillet-chamfer/capture/build.sh target/occt-sdk target/occt-capture-plugin
cargo build --release -p truck-shapeops --example replay_occt_blend
python3 audits/fillet-chamfer/capture/run_capture.py \
  --occt ../investigations/sequential-holes-2026-10-10/occt \
  --library-path target/occt-sdk/usr/lib64 \
  --plugin target/occt-capture-plugin \
  --replay target/release/examples/replay_occt_blend \
  --output target/new-capture --workers 2 --timeout 120
```

Only catalog entries with a recorded native pass and a discovered `blend` or `chamf` token
are selected. `--case 'blend/simple/A*'` narrows the selection. Some lexical hits
never execute a wrapped command and produce no capture. Each native case runs in a fresh
process on an isolated virtual display. Each replay/comparison has its own timeout;
fresh output directories are required. Outputs are observational JSON, never
automatic parity verdicts.

```sh
cargo test -p truck-shapeops --example replay_occt_blend
python3 -m unittest discover -s audits/fillet-chamfer/capture -p 'test_*.py'
```

Twenty-five Rust tests cover physical selection, per-edge fillet radii, equal and
unequal chamfer distances, reference-plane identity, planar distance/angle
conversion on acute, right and obtuse corners, impossible-angle rejection, duplicate
assignments and rejected unsupported/invalid intent. Native-contour regressions
cover expansion, missing scope, seed membership, first-assignment behavior, legacy
labels, unknown schemas, a closed STEP round trip and preserved fillet export
budgets. Ten
Python/Tcl tests cover command selection, native caller scope, interleaved command
numbering, preservation of success/error behavior, and replay of confirmed native
rejections and summaries of complete contour selections. A rejected native
operation has no reference shape: its valid input
is replayed, its native failure is retained separately, and no shape comparison
is attempted. Missing or corrupt evidence remains capture-incomplete.

Chamfer capture schema 2 retains the native mode, seed and every requested
parameter. One native chamfer builder mirrors the command's parameter gates and
first-assignment behavior. Each selection records `native_applied`,
`ignored_reason` and `contour_edges` with complete sampled physical geometry.
Later seeds inside an existing contour are ignored as `already-selected`;
invalid parameter gates and rejected additions are recorded separately.

Replay requires schema-2 scope, matches every contour member, verifies seed
membership and a connected unbranched selection, and deduplicates imported split
edges and compatible assignments. Conflicting active assignments, contradictory
ignored records and missing/empty active scope are rejected explicitly. An
ignored duplicate must belong to a previously applied assignment. No effective
selection is reported as unhandled intent rather than a successful no-op.
Results carry `selection_scope`: `native-contours` for schema-2 chamfers and schema-3 fillets,
`seed-only` for legacy schema 1 (including old hand-written records without a
version). Unknown versions are rejected. Historical
seed-only evidence is not upgraded. Summary manifests retain point counts;
complete raw contour samples remain in the archived input JSON.

Chamfer captures retain every requested distance/angle.
Face-dependent modes also record an analytic reference plane when available.
Replay maps that plane to exactly one adjacent seed face, independently of face
order and plane normal sign, then propagates its side along the oriented contour.
Two-distance replay accepts planar supports and perpendicular plane/cylinder rims;
distance/angle replay uses the oriented angle between planar supports and converts
`d1 = d0 * sin(angle) / sin(beta + angle)` (corner angle `beta`). Perpendicular
plane/cylinder rims use beta = 90 degrees. Consistent unequal setback pairs also
reach the existing wire chamfer route, with side ordering preserved. Unsupported face geometry is reported as unhandled intent. Missing native
data remains missing: the dedicated chamfer fixtures are not replaced by synthetic
probes or counted as passing when only dimension-defined controls are available.

## Final native fillet contours

Fillet schema 3 preserves raw seed radii and records each seed's final contour
index, plus complete contour edges and the final constant radius. A null radius
means the native contour is nonconstant. Capture queries the whole contour after
all `Add` calls: repeated seed assignments can replace radii, and different seed
radii can introduce a variable law. Pre-build per-edge radius queries are not
used because propagated edges can still report zero before native construction.

The capture runner starts a fresh DRAW process and installs the settings observer
before test setup. Successful `tolblend` and `continuityblend` calls are observed
with native numeric conversion; queries and native errors preserve their behavior.
Initial settings mirror the pinned native command's defaults. Native contour
discovery receives those exact settings. Failed observation invalidates further
fillet capture rather than silently using stale settings. This setup contract
does not support attaching the observer to an already configured DRAW session.

Optional R/Q/P modes are captured. Replay currently supports rational C1 fillets,
constant positive contour radii and the existing solid/wire solver routes. It
rejects variable contours, quasi-angular/polynomial representation and C0/C2
internal approximation continuity without replaying a subset. Native tolerances
are retained as evidence; Truck continues to use its independently stated fixed
modeling and export tolerances. These comparisons do not establish equivalence
of the solvers' internal approximation settings. Legacy schema-1 fillets remain
explicitly labeled seed-only. Captured contour membership must cover all active
seeds, and every contour must be connected, unbranched and associated with a seed.

## Stored shape tolerances

Input and native result metadata include `stored_tolerances`, the maximum stored
face, edge and vertex tolerance in model units. STEP comparisons separately
record `native_tolerances` and `truck_step_tolerances`. These are native topology
budgets, not measured geometric errors or certified distance bounds. Reading
them does not change shapes, validity flags, comparison distances or thresholds.
The shared native helper is included in the capture source hashes.

A `native_valid` result is evaluated using the reference shape's own tolerances.
It does not establish accuracy at the replay comparison threshold of 0.001.
For example, the native H4 result has edge/vertex tolerances about 22.97, despite
input tolerances near 1e-6. Conversely, an exact box can have deliberately large
stored tolerances and zero measured boundary difference. Use both diagnostics;
do not turn the stored budget into a synthetic geometry-error measurement.

## Limits

- This route does not yet express unequal-radius corners, variable laws,
  nonrational fillet representations, C0/C2 internal approximation continuity,
  open shells or multiple solids. Face-dependent chamfer
  contours still require a planar seed reference and either two planar supports
  or a perpendicular plane/cylinder rim at every matched edge. Rejection here does not prove no equivalent Truck route exists.
- STEP conversion and closed-edge healing can fail before a blend is attempted.
  Those failures require separate diagnosis and are not fillet solver verdicts.
- Selection matching and output comparison use finite samples, not certified
  Hausdorff bounds. They do not establish G1 continuity, global self-intersection,
  editing/history stability or practical latency. All comparison statuses remain
  unverified; measured differences are retained for investigation.
- Modeling tolerance is 0.001, selection matching 0.0001, tessellation 0.001 and
  chamfer STEP fitting 0.0000001, and fillet STEP fitting 0.00005. Each structured
  replay result records its actual `step_fitting_tolerance`. Chamfer fits stay
  below kernel consistency tolerance; A4 checks a strict exported/reimported
  solid. Fillets retain their supported subdivision budget and the exporter's
  robust STEP tessellation contract; D4 checks that round trip with the shared
  STEP harness. Native test tolerances and model scale may differ.
- Per-process wall times include instrumentation and contention; they are not
  comparative performance benchmarks. The installed native binary is not a build
  of the locally modified OCCT source checkout.

The first instrumented baseline and its observed gaps are documented in
`baseline/REPORT.md`. Later kernel fixes do not retroactively alter those results.

## External native test data

Both `run_occt.py` and `capture/run_capture.py` accept repeatable `--data-path DIR`.
The default OCCT data directory stays first. Additional directories and every file's
SHA-256 are recorded in `test_data`; the runner checks them again before marking
`test_data_verified` true. Keep the directories unchanged throughout execution.
For native tests whose static catalog says missing-in-checkout, use
`--include-missing` with the additional directory. Run the unchanged native cases
and rebuild the catalog before capture: only native-passing entries are selected.
Dedicated chamfer grids are selected even when their Tcl helpers build commands
dynamically. External data does not establish a passing test or Truck parity.
