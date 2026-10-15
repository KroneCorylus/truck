# Complete native chamfer contour intent

Parent aa3611024339c9b8a6879542023f7dac5301f721. The native capture previously saved only
seed edges. A4 can construct a valid one-edge chamfer while missing the native
three-edge contour, so construction success alone did not establish equivalent
intent. Historical seed-only records remain unchanged and explicitly limited.

Chamfer schema 2 uses one native builder, mirrors parameter gates and preserves
first-assignment behavior. It retains raw seeds/parameters, applied/ignored state
and all propagated physical contour edges. Replay verifies seed membership,
complete connected scope, ignored-record consistency and assignment conflicts.
Missing scope cannot fall back to a seed; no effective selection is explicit.
Legacy chamfers and fillets remain labeled seed-only. Curved reference-face
propagation is still unsupported by the planar reference mapping.

Nine Rust regressions fail on the parent: eight scope/assignment/label checks
and one explicit complete-contour STEP round trip. A Python summary regression
also fails on the parent. Final validation: 24 Rust replay tests, 22 audit tests,
10 capture tests, all-target release Clippy, action smoke, native plugin build
and the final replay build pass. Kernel sources did not change; their preceding
504-pass/0-failure/1-existing-ignore release validation is linked in metadata.

The initial round-trip test exposed about 6.18e-6 contact drift allowed by the
old replay STEP fit of 5e-5. The exporter documents robust tessellation for such
fits. Replay now requests 1e-7, below kernel consistency tolerance; the original
strict solid and STEP assertions pass without relaxing them. The legacy label
test does not compare a seed-only partial edit to the full-contour material
oracle. Complete three-edge results always run the shared solid/STEP harness.
The general prepare_for_step API and its tolerance contract are unchanged.

Eight real native controls verify first-assignment behavior in equal-distance,
two-distance and distance-angle modes; invalid-parameter gates; no-op rejection;
legacy fillet scope; and ignoring a different seed inside A4's already selected
contour. Seven construct and meet native-valid single-solid STEP and sampled
comparison checks. The invalid-only command is natively rejected and replayed as
unhandled intent. Box volumes use independent section formulas. Box and A4 input
BREP bytes are identical before/after the native control sequence.

Automatic A4/001 now captures and replays three edges. Native-valid single-solid
STEP, immutable input, consistent geometry and closed mesh all pass. Maximum
sampled boundary distance is 2.2291002058941381e-08;
native volume 0.5630665232959563, Truck STEP volume 0.56306652329136542.
This is finite sampled evidence, not a global bound or full-parity verdict.
The remaining seven A4 sequence inputs are retained with their separate results;
they are not counted as successful merely because the native case passes.

Native evidence archive: 225 byte-verified members, SHA256
be4b37516d1322497609101e75e594d4ec76eb8226d6d06ac62c951fde09f2a0. Raw target/native-chamfer-contour-controls and
target/occt-capture-native-chamfer-contour-proof. Source/executable hashes were
verified after the final replay build. No performance claim.

A clean full corpus capture follows this implementation commit. Its comparison
must distinguish corrected native intent from unchanged intent, preserve all
prior successful checks for unchanged intent, and record numerical changes from
tighter STEP fitting. Remaining work includes unsupported curved reference
faces, solver failures, native shell inputs, C1 input fidelity, missing fixtures,
sampled discrepancies, fillet tangent propagation, editing/continuity evidence
and the previously recorded PCurve precision failure.
