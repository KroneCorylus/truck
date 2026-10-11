# Physical blend capture and replay

This tool records actual input solids, requested radii and physical edge curves
from OCCT's `blend` command, then tries equivalent Truck constructions. It does
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

Only catalog entries with a recorded native pass and a discovered `blend` token
are selected. `--case 'blend/simple/A*'` narrows the selection. Some lexical hits
never execute `blend` and produce no capture. Each native case runs in a fresh
process on an isolated virtual display. Each replay/comparison has its own timeout;
fresh output directories are required. Outputs are observational JSON, never
automatic parity verdicts.

```sh
cargo test -p truck-shapeops --example replay_occt_blend
python3 -m unittest discover -s audits/fillet-chamfer/capture -p 'test_*.py'
```

Five Rust tests cover split/reversed/repeated physical selections, incomplete
and ambiguous matches, invalid points, branches and disconnected wires. Two Tcl
wrapper tests check caller scope and preservation of success/error behavior.

## Limits

- This route does not yet express unequal-radius corners, variable laws, tangent
  expansion, disconnected contour sequencing, open shells, multiple solids or
  chamfer commands. Rejection here does not prove no equivalent Truck route exists.
- STEP conversion and closed-edge healing can fail before a blend is attempted.
  Those failures require separate diagnosis and are not fillet solver verdicts.
- Selection matching and output comparison use finite samples, not certified
  Hausdorff bounds. They do not establish G1 continuity, global self-intersection,
  editing/history stability or practical latency. All comparison statuses remain
  unverified; measured differences are retained for investigation.
- Modeling tolerance is 0.001, selection matching 0.0001, tessellation 0.001 and
  STEP fitting 0.00005. Native test tolerances and model scale may differ.
- Per-process wall times include instrumentation and contention; they are not
  comparative performance benchmarks. The installed native binary is not a build
  of the locally modified OCCT source checkout.

The first instrumented baseline and its observed gaps are documented in
`baseline/REPORT.md`. Later kernel fixes do not retroactively alter those results.
