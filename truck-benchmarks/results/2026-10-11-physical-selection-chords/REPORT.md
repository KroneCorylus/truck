# Physical selection coverage between sampled vertices

A captured parabolic selection could incorrectly match its straight chord because
curve subdivision represents a line by endpoints only. The old check accepted
both edges and misclassified G1 as a nontangent closed wire. Matching now verifies
complete candidate polyline chords against the captured polyline's tolerance tube.

Each chord is accepted when both endpoints lie in one segment capsule (convexity),
or when endpoint distances and chord length establish a 1-Lipschitz bound. Remaining
chords subdivide; uncertainty at depth/precision limits rejects the match. A 1e-5
margin is reserved for candidate curve subdivision inside the total 1e-4 matching
tolerance. Actual curve subdivision and the reverse selection-coverage check remain
finite approximations; this is not full analytic shape certification.

Both new tests fail on the parent: a parabola also selects its straight chord, and
a bent path sharing endpoints/midpoint with a line is falsely ambiguous. All five
Rust replay tests and final all-target Clippy pass. Existing split/reversed,
incomplete, ambiguous and branched-selection tests remain unchanged. Failure reports
now include the selected edge count. No production kernel behavior changes.

A fresh three-case replay preserves the native successes. G1 now reaches the
fillet face-trimming stage and reports BlendConstructionFailed, rather than the
false nontangent-wire diagnosis. F9's non-tangent two-edge corner and G2's mixed
selection remain unsupported by these replay routes. These are unresolved modeling
tasks. The exact capture is retained in `target/occt-capture-chord-coverage`; a full
replay with final kernel/matcher sources is next. No speed or full-parity claim is made.
