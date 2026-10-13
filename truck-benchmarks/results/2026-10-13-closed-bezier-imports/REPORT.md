# Closed Bézier prism import and boundary charts

Recover the cubic and quartic closed-profile STEP prisms represented by native cases `bug30940_2` through `_6`. A cusp has coincident profile endpoints with distinct tangents; it is not a periodic curve. The first boundary point needs the same parameter branch as its following edge, including when an entire initial edge lies on the ambiguous vertical seam.

`boundary_search_hint` supplies that branch to both face preprocessing and tessellation, without changing boundary order or supplied spatial points. Only coincident ends of a nonperiodic parameter range with noncollapsed tangents trigger the continuation seed. Coincidence uses modeling tolerance so fitted seam roundoff is retained; candidate inverses must still satisfy the finite domain and point-residual checks. The seam scan follows the previous inverse before retrying a global search. The existing closed spline-pipe regression remains unchanged and passes. NURBS curve inclusion retries a failed continuation without a hint, preserving six stations per curve degree in every unique knot span, geometric residual checks and the bounded surface domain. The two 3D inclusion implementations share the same traversal.

Parent regressions are recorded against `d4115121`: two inclusion tests fail, the mesher selects the wrong half-domain, and both native-input tests fail with `NotSimpleWire`. The negative inclusion regression already passes and remains unchanged. No existing assertion or tolerance was weakened.

The input-solid regression covers both profiles, strict/robust preprocessing, three original boundary starts, rigid transforms and scale 0.1. It checks geometric consistency, 4 vertices/6 edges/4 faces, independent volumes 150 and 7200/7, closed tessellation, and STEP round trips. The mesh regression covers 96 charts across cubic/quartic cusps and a complete rational ellipse, both half-domains, nonuniform affine transforms, both orientations and four boundary starts; original spatial points, UV bounds, signed area and winding are checked. Inclusion tests also retain rejection of out-of-domain curves and an interior off-surface excursion in a later knot span.

All five targeted native replays reach blending with unchanged inputs on failure. `_2` and `_5` reject the nontangent closed rim; `_3` cannot solve its rolling contacts; `_4` and `_6` require nonplanar multi-edge corner support. **No fillet recovery is claimed by this batch.**

Independent OCCT checks of the recovered input exports:

| Input | Valid / single solid | Exact volume | Absolute volume error | Sampled native → Truck | Sampled Truck → native |
| --- | --- | ---: | ---: | ---: | ---: |
| Cubic | Yes / yes | 150 | 7.731e-12 | 7.235e-13 | 3.204e-15 |
| Quartic | Yes / yes | 7200/7 | 1.592e-12 | 3.845e-14 | 6.793e-14 |

The largest edge/vertex tolerance in both exports is 1e-7. Distance comparisons are sampled evidence, not certified global bounds. The exact volumes derive from the profile control points in the fixture README. The raw native bytes come from the archived conical-domain capture; provenance and output hashes are recorded in the metadata and replay archive.

Release-suite, all-target Clippy, benchmark smoke and replay-build outcomes are recorded in the adjacent logs and metadata. No dependency or breaking API change, and no performance claim.

Final six-crate release validation: **964 passed, one unchanged PCurve precision failure, six existing ignored tests, 155 targets**. The failure is `truck-geometry/tests/pcurve.rs::pcurve_derivation`, with the same -2829663063.761872 / -2829663063.7618704 values as the committed parent proof; no new failure occurred. All-target release Clippy (including benchmarks), the single-threaded action smoke, and the final production replay build passed.

All five targeted replays and both independent input checks were rerun using the final source and replay executable. The targeted archive contains **50 byte-verified members**; SHA-256 `f253c1707dacfe463d68a79b3467cb93e1b2d0d2f9c5e0be2ea2a5d3e11dac7a`. The replay executable SHA-256 is `96c74ac328a49af7d755572f4664923c3fb966378849101a8aeb3beb7d06c028`.
